use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CStr, CString},
    fs,
    io::Cursor,
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{to_string, CSlice, Handler, LogLevel},
    embedder::interface::{DynamicEmbedderV1, DYNAMIC_EMBEDDER_ABI_VERSION_V1, INVALID_HANDLE},
    interface::PluginInit,
};
use base64::prelude::*;
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use image::{ImageFormat, ImageReader};
use mime_guess::{mime, Mime};
use schemars::schema_for;

use crate::param::Param;

pub mod param;

static PLUGIN: Mutex<Option<DynamicEmbedderV1>> = Mutex::new(None);
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
pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicEmbedderV1 {
    let plugin_read = unsafe { init.read().clone() };
    unsafe {
        std::ptr::write(std::ptr::addr_of_mut!(LOGGER), Some(plugin_read.log));
    }

    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(plugin_read);

        let new_plugin = DynamicEmbedderV1 {
            abi_version: DYNAMIC_EMBEDDER_ABI_VERSION_V1,
            struct_size: std::mem::size_of::<DynamicEmbedderV1>(),
            get_param_schema,
            get_handler,
            free_handler,
            embed,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = plugin_mtx.as_ref().unwrap();
    return &raw const *ret;
}

struct HandleInfo {
    pub param: Param,
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

unsafe extern "C" fn get_handler(params: *const c_char) -> Handler {
    let param_str = to_string(params);
    let param_result = serde_json::from_str::<Param>(&param_str).map_err(|err| err.to_string());
    if param_result.is_ok() {
        let mut handler_mtx = HANDLER_COUNTER.lock().unwrap();
        let handler = (*handler_mtx).clone();
        (*handler_mtx) += 1;

        let info = HandleInfo {
            param: param_result.unwrap(),
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

struct EmbeddingRef {
    _embedding: Box<Vec<f32>>,
    ret: Box<CSlice<f32>>,
}

unsafe impl Send for EmbeddingRef {}
unsafe impl Sync for EmbeddingRef {}

static EMBEDDING_REF: LazyLock<Mutex<HashMap<usize, EmbeddingRef>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

unsafe extern "C" fn embed(handler: Handler, file_path: *const c_char) -> *const CSlice<f32> {
    let mut mtx = HANDLERS.lock().unwrap();
    let info_opt = mtx.get_mut(&handler);

    if let Some(info) = info_opt {
        let param = &info.param;
        let file_path_str = CStr::from_ptr(file_path).to_str().unwrap();
        let (extracted, mime_type) = extract_from(file_path_str, ImageFormat::Png);
        if extracted.is_none() {
            log(
                LogLevel::Warn,
                &format!("extraction failed from {file_path_str}"),
            );
            return std::ptr::null();
        }
        let api_param = param.get_api_param(&mime_type);
        if api_param.is_none() {
            log(LogLevel::Warn, &format!("processable api param not found"));
            return std::ptr::null();
        }

        let embedding_opt = match extracted.unwrap() {
            ExtractType::Text(content) => api_param.unwrap().try_get_embedding(Some(content), None),
            ExtractType::ImageBase64(content) => {
                api_param.unwrap().try_get_embedding(None, Some(content))
            }
        };
        if embedding_opt.is_err() {
            log(LogLevel::Warn, &embedding_opt.unwrap_err());
            return std::ptr::null();
        }

        let embedding = Box::new(embedding_opt.unwrap());
        let ret_ref = Box::new(CSlice {
            ptr: embedding.as_ptr(),
            length: embedding.len(),
        });
        let ret_ref = EmbeddingRef {
            _embedding: embedding,
            ret: ret_ref,
        };
        let ret_ptr: *const CSlice<f32> = &*ret_ref.ret;
        EMBEDDING_REF
            .lock()
            .unwrap()
            .insert(ret_ptr as usize, ret_ref);
        return ret_ptr;
    }
    return std::ptr::null();
}

unsafe extern "C" fn free(value: *const c_void) {
    STR_REF.lock().unwrap().remove(&(value as usize));
    EMBEDDING_REF.lock().unwrap().remove(&(value as usize));
}

enum ExtractType {
    Text(String),
    ImageBase64(String),
}

fn extract_from(file_path: &str, image_format: ImageFormat) -> (Option<ExtractType>, Mime) {
    let guess = mime_guess::from_path(file_path);
    let mime = guess.first_or_octet_stream();
    match mime.type_() {
        mime::TEXT => {
            let file = fs::read(file_path);
            if file.is_err() {
                log(LogLevel::Error, &file.unwrap_err().to_string());
                return (None, mime);
            }
            let file_bin = file.unwrap();
            let content = decode_to_string(&file_bin);
            (Some(ExtractType::Text(content)), mime)
        }
        mime::IMAGE => {
            let img = ImageReader::open(file_path);
            if let Err(err) = img {
                log(LogLevel::Error, &err.to_string());
                return (None, mime);
            }
            let reader = img.unwrap();
            let decoded = reader.decode();
            if let Err(err) = decoded {
                log(LogLevel::Error, &err.to_string());
                return (None, mime);
            }
            let decoded_img = decoded.unwrap();
            let mut output = Cursor::new(Vec::<u8>::new());
            let write = decoded_img.write_to(&mut output, image_format);
            if let Err(err) = write {
                log(LogLevel::Error, &err.to_string());
                return (None, mime);
            }

            let encoded = BASE64_STANDARD.encode(output.into_inner());
            (Some(ExtractType::ImageBase64(encoded)), mime)
        }
        _ => (None, mime),
    }
}

fn decode_to_string(data: &[u8]) -> String {
    let encoding = {
        let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);

        detector.feed(data, true);
        detector.guess(None, Utf8Detection::Allow)
    };

    let (text, _, had_errors) = encoding.decode(data);

    if had_errors {
        let name = encoding.name();
        log(
            LogLevel::Warn,
            &format!("Warning: {name} としてデコードしましたが、不正なバイト列があります"),
        );
    }

    text.into_owned()
}
