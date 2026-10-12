use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{Handler, LogLevel},
    embedder::params::*,
};
use candle_core::{DType, Device};
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use fastembed::Qwen3VLEmbedding;
use image::{ImageFormat, ImageReader};
use mime_guess::{mime, Mime};
use schemars::schema_for;
use uuid::Uuid;

use crate::{
    log,
    param::{BackendDevice, Param},
};

struct HandleInfo {
    pub _param: Param,
    pub model: Qwen3VLEmbedding,
}

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

pub fn get_param_schema(_args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
    let schema = schema_for!(Param);
    return Ok(GetParamSchemaData {
        schema: serde_json::to_value(&schema).map_err(|err| err.to_string())?,
    });
}

pub fn get_handler(args: GetHandlerArgs) -> Result<GetHandlerData, String> {
    let param_result =
        serde_json::from_value::<Param>(args.params).map_err(|err| err.to_string())?;
    let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
    let handler = (*handler_mtx).clone();
    (*handler_mtx) += 1;

    let device = get_device(&param_result.device)?;
    let model = Qwen3VLEmbedding::from_hf(
        "Qwen/Qwen3-VL-Embedding-2B",
        &device,
        DType::F32,
        param_result.dimensions as usize,
    )
    .map_err(|err| err.to_string())?;

    let info = HandleInfo {
        _param: param_result,
        model,
    };

    HANDLERS.lock().unwrap().insert(handler, info);
    Ok(GetHandlerData { handler: handler })
}

pub fn free_handler(args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
    HANDLERS.lock().unwrap().remove(&args.handler);
    Ok(FreeHandlerData {})
}

pub fn embed(args: EmbedArgs) -> Result<EmbedData, String> {
    let handler = args.handler;
    let mut mtx = HANDLERS.lock().unwrap();
    let info = mtx
        .get_mut(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let file_path_str = &args.file_path;
    let source_path = Path::new(file_path_str);
    let image_output_path = source_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.png", Uuid::new_v4()));

    let result = (|| {
        let (extracted_opt, _mime_type) =
            extract_from(file_path_str, &image_output_path, ImageFormat::Png)?;
        let extracted = extracted_opt.ok_or(format!("extraction failed from {file_path_str}"))?;

        let model = &info.model;

        let embedding = match extracted {
            ExtractType::Text(content) => model
                .embed_texts(&[content])
                .map_err(|err| err.to_string())?,
            ExtractType::ImagePngPath => model
                .embed_images(&[&image_output_path])
                .map_err(|err| err.to_string())?,
        };

        Ok(EmbedData {
            embeddings: embedding
                .get(0)
                .ok_or_else(|| "embedding not yielded".to_string())?
                .to_vec(),
        })
    })();

    match fs::remove_file(&image_output_path) {
        Ok(()) => result,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => result,
        Err(cleanup_err) => match result {
            Ok(_) => Err(format!(
                "failed to remove temporary image {}: {cleanup_err}",
                image_output_path.display()
            )),
            Err(processing_err) => Err(format!(
                "{processing_err}; failed to remove temporary image {}: {cleanup_err}",
                image_output_path.display()
            )),
        },
    }
}

fn get_device(backend: &BackendDevice) -> Result<Device, String> {
    match backend {
        BackendDevice::Cpu => Ok(Device::Cpu),
        BackendDevice::Cuda => {
            #[cfg(feature = "cuda")]
            {
                let device = Device::cuda_if_available(0).map_err(|err| err.to_string())?;
                if device.is_cpu() {
                    return Err("CUDA was requested, but no CUDA device is available".into());
                }
                Ok(device)
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err("CUDA support is not enabled; rebuild with --features cuda".into())
            }
        }
        BackendDevice::Metal => {
            #[cfg(all(feature = "metal", target_os = "macos"))]
            {
                let device = Device::metal_if_available(0).map_err(|err| err.to_string())?;
                if device.is_cpu() {
                    return Err("Metal was requested, but no Metal device is available".into());
                }
                Ok(device)
            }
            #[cfg(all(feature = "metal", not(target_os = "macos")))]
            {
                Err("Metal support is only available on macOS".into())
            }
            #[cfg(not(feature = "metal"))]
            {
                Err("Metal support is not enabled; rebuild with --features metal".into())
            }
        }
    }
}

enum ExtractType {
    Text(String),
    ImagePngPath,
}

fn extract_from(
    file_path: &str,
    image_output_path: &Path,
    image_format: ImageFormat,
) -> Result<(Option<ExtractType>, Mime), String> {
    let guess = mime_guess::from_path(file_path);
    let mime = guess.first_or_octet_stream();
    match mime.type_() {
        mime::TEXT => {
            let file_bin = fs::read(file_path).map_err(|err| err.to_string())?;
            let encoding = {
                let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);

                detector.feed(&file_bin, true);
                detector.guess(None, Utf8Detection::Allow)
            };

            let (text, _, had_errors) = encoding.decode(&file_bin);

            if had_errors {
                let enc = encoding.name();
                log(
                    LogLevel::Warn,
                    &format!("{file_path} is decoded as {enc} and got error"),
                );
            }

            Ok((Some(ExtractType::Text(text.into_owned())), mime))
        }
        mime::IMAGE => {
            let reader = ImageReader::open(file_path).map_err(|err| err.to_string())?;
            let decoded_img = reader.decode().map_err(|err| err.to_string())?;
            let mut output = Cursor::new(Vec::<u8>::new());
            decoded_img
                .write_to(&mut output, image_format)
                .map_err(|err| err.to_string())?;

            fs::write(image_output_path, output.into_inner()).map_err(|err| err.to_string())?;

            Ok((Some(ExtractType::ImagePngPath), mime))
        }
        _ => Err(format!("unsupported mime: {mime}")),
    }
}
