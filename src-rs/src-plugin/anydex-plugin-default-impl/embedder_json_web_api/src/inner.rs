use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{Handler, LogLevel},
    embedder::params::*,
};
use base64::prelude::*;
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use image::{ImageFormat, ImageReader};
use mime_guess::{mime, Mime};
use schemars::schema_for;

use crate::{log, param::Param};

struct HandleInfo {
    pub param: Param,
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

    let info = HandleInfo {
        param: param_result,
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

    let param = &info.param;
    let file_path_str = &args.file_path;
    let (extracted_opt, mime_type) = extract_from(file_path_str, ImageFormat::Png)?;
    let extracted = extracted_opt.ok_or(format!("extraction failed from {file_path_str}"))?;

    let api_param = param
        .get_api_param(&mime_type)
        .ok_or(format!("processable api param not found"))?;

    let embedding = match extracted {
        ExtractType::Text(content) => api_param.try_get_embedding(Some(content), None),
        ExtractType::ImageBase64(content) => api_param.try_get_embedding(None, Some(content)),
    }?;

    return Ok(EmbedData {
        embeddings: embedding,
    });
}

enum ExtractType {
    Text(String),
    ImageBase64(String),
}

fn extract_from(
    file_path: &str,
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

            let encoded = BASE64_STANDARD.encode(output.into_inner());
            Ok((Some(ExtractType::ImageBase64(encoded)), mime))
        }
        _ => Err(format!("unsupported mime: {mime}")),
    }
}
