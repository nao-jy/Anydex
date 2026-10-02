use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CStr, CString},
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{to_string, CSlice, Handler, LogLevel},
    file_extractor::interface::{
        DynamicFileExtractorV1, DYNAMIC_FILE_EXTRACTOR_ABI_VERSION_V1, INVALID_HANDLE,
    },
    interface::PluginInit,
};
use mime_guess::mime;
use schemars::schema_for;
use tokio::runtime::Runtime;
use uuid::Uuid;
use xberg::{
    extract as xberg_extract, ChunkerType, ChunkingConfig, ExtractInput, ExtractionConfig,
    OcrConfig,
};

use crate::param::Param;

pub mod param;

static PLUGIN: Mutex<Option<DynamicFileExtractorV1>> = Mutex::new(None);
static PLUGIN_INIT: Mutex<Option<PluginInit>> = Mutex::new(None);

static mut LOGGER: Option<unsafe extern "C" fn(log_level: LogLevel, message: *const c_char)> = None;

fn log(level: LogLevel, msg: &str) {
    let logger = unsafe { std::ptr::read_volatile(std::ptr::addr_of!(LOGGER)) };
    let Some(logger_fn) = logger else {
        return;
    };

    let cmsg = CString::new(msg).unwrap();
    unsafe { logger_fn(level, cmsg.as_ptr()) };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicFileExtractorV1 {
    let plugin_read = unsafe { init.read().clone() };
    unsafe {
        std::ptr::write(std::ptr::addr_of_mut!(LOGGER), Some(plugin_read.log));
    }

    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(plugin_read);

        let new_plugin = DynamicFileExtractorV1 {
            abi_version: DYNAMIC_FILE_EXTRACTOR_ABI_VERSION_V1,
            struct_size: std::mem::size_of::<DynamicFileExtractorV1>(),
            get_param_schema,
            get_handler,
            free_handler,
            extract,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = plugin_mtx.as_ref().unwrap();
    return &raw const *ret;
}

struct HandleInfo {
    pub param: Param,
    pub base_dir: String,
    pub rt: Runtime,
}

static STR_REF: LazyLock<Mutex<HashMap<usize, Box<CString>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLERS: LazyLock<Mutex<HashMap<Handler, HandleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLER_COUNTER: Mutex<u64> = Mutex::new(0);

unsafe extern "C" fn get_param_schema() -> *const c_char {
    let schema = schema_for!(Param);
    let schema_str: Box<CString> =
        Box::from(CString::new(serde_json::to_string_pretty(&schema).unwrap()).unwrap());
    let ptr = schema_str.as_ptr();
    STR_REF.lock().unwrap().insert(ptr as usize, schema_str);

    return ptr;
}

unsafe extern "C" fn get_handler(params: *const c_char, base_dir: *const c_char) -> Handler {
    let param_str = to_string(params);
    let param_result = serde_json::from_str::<Param>(&param_str).map_err(|err| err.to_string());
    if param_result.is_ok() {
        let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
        let handler = (*handler_mtx).clone();
        (*handler_mtx) += 1;

        let info = HandleInfo {
            param: param_result.unwrap(),
            base_dir: to_string(base_dir),
            rt: Runtime::new().unwrap(),
        };

        HANDLERS.lock().unwrap().insert(handler, info);
        handler
    } else {
        log(LogLevel::Error, &param_result.unwrap_err());
        INVALID_HANDLE
    }
}

unsafe extern "C" fn free_handler(handler: Handler) {
    HANDLERS.lock().unwrap().remove(&handler);
}

struct ExtractRef {
    _chars: Vec<Box<CString>>,
    _vec: Box<Vec<*const c_char>>,
    ret: Box<CSlice<*const c_char>>,
}

unsafe impl Send for ExtractRef {}
unsafe impl Sync for ExtractRef {}

static EXTRACT_REF: LazyLock<Mutex<HashMap<usize, ExtractRef>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

unsafe extern "C" fn extract(
    handler: Handler,
    file_abs_path: *const c_char,
) -> *const CSlice<*const c_char> {
    let mut mtx = HANDLERS.lock().unwrap();
    let info_opt = mtx.get_mut(&handler);

    if let Some(info) = info_opt {
        let rt = &info.rt;
        let param = &info.param;
        let base_dir = &info.base_dir;

        let mut chars: Vec<Box<CString>> = Vec::new();

        let file_path = CStr::from_ptr(file_abs_path).to_string_lossy().to_string();
        let guess = mime_guess::from_path(&file_path);
        let mime = guess.first_or_octet_stream();
        let copy_result = match mime.type_() {
            mime::IMAGE => {
                let file_stem = Uuid::new_v4().to_string();
                let ext = Path::new(&file_path).extension().unwrap().to_string_lossy();
                let filename = format!("{file_stem}.{ext}");
                let copy_path = Path::new(base_dir).join(&filename);
                std::fs::copy(&file_path, copy_path)
            }
            _ => Ok(0u64),
        };
        if let Err(err) = &copy_result {
            log(LogLevel::Error, &err.to_string());
        }

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
            log(LogLevel::Error, &err.to_string());
        }

        let output_unwrapped = output.unwrap();

        for err in &output_unwrapped.errors {
            let code: u32 = err.code;
            let msg = &err.message;
            log(LogLevel::Warn, &format!("{code}: {msg}"));
        }

        let extract_result = &output_unwrapped.results;
        for result in extract_result {
            // output text
            if let Some(chunks) = &result.chunks {
                for chunk in chunks {
                    let name = Uuid::new_v4().to_string();
                    let filename = format!("{name}.txt");

                    let write_result =
                        fs::write(Path::new(base_dir).join(&filename), &chunk.content);

                    if let Err(err) = write_result {
                        log(LogLevel::Warn, &err.to_string());
                    } else if let Ok(_) = write_result {
                        chars.push(Box::new(CString::new(filename).unwrap()));
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
                        chars.push(Box::new(CString::new(filename).unwrap()));
                    }
                }
            }
        }

        let vec: Box<Vec<*const i8>> = Box::new(chars.iter().map(|c| c.as_ptr()).collect());
        let slice = CSlice {
            ptr: vec.as_ptr(),
            length: vec.len(),
        };
        let ext_ref = ExtractRef {
            _chars: chars,
            _vec: vec,
            ret: Box::new(slice),
        };
        let ret_ptr: *const CSlice<*const c_char> = &*ext_ref.ret;
        EXTRACT_REF
            .lock()
            .unwrap()
            .insert(ret_ptr as usize, ext_ref);
        return ret_ptr;
    }
    return std::ptr::null();
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
    EXTRACT_REF.lock().unwrap().remove(&(value as usize));
}
