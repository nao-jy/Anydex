use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{Handler, LogLevel},
    file_extractor::params::*,
};
use mime_guess::mime;
use schemars::schema_for;
use tokio::runtime::Runtime;
use uuid::Uuid;
use xberg::{
    extract as xberg_extract, ChunkerType, ChunkingConfig, ExtractInput, ExtractionConfig,
    OcrConfig, XbergError,
};

use crate::{log, param::Param};

struct HandleInfo {
    pub param: Param,
    pub base_dir: String,
    pub rt: Runtime,
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
        base_dir: args.base_dir,
        rt: Runtime::new().unwrap(),
    };

    HANDLERS.lock().unwrap().insert(handler, info);
    Ok(GetHandlerData { handler })
}

pub fn free_handler(args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
    HANDLERS.lock().unwrap().remove(&args.handler);
    Ok(FreeHandlerData {})
}

pub fn extract(args: ExtractArgs) -> Result<ExtractData, String> {
    let handler = args.handler;
    let mut mtx = HANDLERS.lock().unwrap();
    let info = mtx
        .get_mut(&handler)
        .ok_or(format!("handler {handler} not found"))?;

    let rt = &info.rt;
    let param = &info.param;
    let base_dir = &info.base_dir;
    let file_path = &args.file_path;

    let guess = mime_guess::from_path(file_path);
    let mime = guess.first_or_octet_stream();
    let _copy_result = match mime.type_() {
        mime::IMAGE => {
            let file_stem = Uuid::new_v4().to_string();
            let ext = Path::new(file_path).extension().unwrap().to_string_lossy();
            let filename = format!("{file_stem}.{ext}");
            let copy_path = Path::new(base_dir).join(&filename);
            std::fs::copy(file_path, copy_path)
        }
        _ => Ok(0u64),
    }
    .map_err(|err| err.to_string())?;

    let config = ExtractionConfig {
        ocr: Some(OcrConfig {
            enabled: true,
            language: param.ocr_languages.clone(),
            ..OcrConfig::default()
        }),
        chunking: Some(ChunkingConfig {
            max_characters: param.text_window,
            overlap: param.text_overwrap,
            trim: true,
            chunker_type: ChunkerType::Semantic,

            ..ChunkingConfig::default()
        }),
        ..ExtractionConfig::default()
    };
    let output = rt.block_on(xberg_extract(ExtractInput::from_uri(file_path), &config));

    if let Err(err) = &output {
        match err {
            XbergError::UnsupportedFormat(_) => {
                return Ok(ExtractData {
                    is_supported: false,
                    extracted_file_paths: vec![],
                })
            }
            _ => return Err(err.to_string()),
        }
    }

    let output = output.unwrap();

    for err in &output.errors {
        let code: u32 = err.code;
        let msg = &err.message;
        log(LogLevel::Warn, &format!("{code}: {msg}"));
    }

    let extract_result = &output.results;
    let mut ret = Vec::new();
    for result in extract_result {
        // output text
        if let Some(chunks) = &result.chunks {
            for chunk in chunks {
                let name = Uuid::new_v4().to_string();
                let filename = format!("{name}.txt");

                let write_result = fs::write(Path::new(base_dir).join(&filename), &chunk.content);

                if let Err(err) = write_result {
                    log(LogLevel::Warn, &err.to_string());
                } else if let Ok(_) = write_result {
                    ret.push(filename);
                }
            }
        }

        // output image
        let images_opt = &result.images;
        if let Some(imgages) = images_opt {
            for image in imgages {
                let name = Uuid::new_v4().to_string();
                let format = &image.format;
                let filename = format!("{name}.{format}");

                let write_result = fs::write(Path::new(base_dir).join(&filename), &image.data);

                if let Err(err) = write_result {
                    log(LogLevel::Warn, &err.to_string());
                } else if let Ok(_) = write_result {
                    ret.push(filename);
                }
            }
        }
    }

    return Ok(ExtractData {
        extracted_file_paths: ret,
        is_supported: true,
    });
}
