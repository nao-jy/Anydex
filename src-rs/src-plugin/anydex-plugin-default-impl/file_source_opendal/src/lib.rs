use std::{
    collections::HashMap,
    ffi::{c_char, CString},
    sync::{LazyLock, Mutex},
};

use anydex_plugin_api_lib::{
    common::{result_to_api_result, to_string, APIResult, APIResultSerialized, LogLevel},
    file_source::{
        interface::{DynamicFileSourceV1, DYNAMIC_FILE_SOURCE_ABI_VERSION_V1},
        params::*,
    },
    interface::PluginInit,
};

pub mod gdrive;
pub mod inner;
pub mod param;

static PLUGIN: Mutex<Option<DynamicFileSourceV1>> = Mutex::new(None);
static PLUGIN_INIT: Mutex<Option<PluginInit>> = Mutex::new(None);

static mut LOGGER: Option<unsafe extern "C" fn(log_level: LogLevel, message: *const c_char)> = None;

fn _log(level: LogLevel, msg: &str) {
    let logger = unsafe { std::ptr::read_volatile(std::ptr::addr_of!(LOGGER)) };
    let Some(logger_fn) = logger else {
        return;
    };

    let cmsg = CString::new(msg).unwrap();
    unsafe { logger_fn(level, cmsg.as_ptr()) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_plugin(init: *const PluginInit) -> *const DynamicFileSourceV1 {
    let plugin_read = unsafe { init.read().clone() };
    unsafe {
        std::ptr::write(std::ptr::addr_of_mut!(LOGGER), Some(plugin_read.log));
    }

    let mut plugin_mtx = PLUGIN.lock().unwrap();
    if (*plugin_mtx).is_none() {
        let mut plugin_init_mtx = PLUGIN_INIT.lock().unwrap();
        *plugin_init_mtx = Some(plugin_read);

        let new_plugin = DynamicFileSourceV1 {
            abi_version: DYNAMIC_FILE_SOURCE_ABI_VERSION_V1,
            struct_size: std::mem::size_of::<DynamicFileSourceV1>(),
            get_param_schema,
            get_handler,
            free_handler,
            get_diff,
            download,
            free,
        };
        *plugin_mtx = Some(new_plugin);
    }

    let ret = plugin_mtx.as_ref().unwrap();
    return &raw const *ret;
}

static STR_REF: LazyLock<Mutex<HashMap<usize, Box<CString>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

macro_rules! define_api_callback {
    ($name:ident, $args:ty) => {
        unsafe extern "C" fn $name(args_serialized: *const c_char) -> APIResultSerialized {
            let args_str = to_string(args_serialized);
            let ret = match serde_json::de::from_str::<$args>(&args_str) {
                Ok(args_ok) => result_to_api_result(inner::$name(args_ok)),
                Err(err) => APIResult::Err {
                    error: err.to_string(),
                },
            };
            let ret_ser = serde_json::to_string(&ret).unwrap();
            let ret_ptr: Box<CString> = Box::from(CString::new(ret_ser).unwrap());
            let ptr = ret_ptr.as_ptr();
            STR_REF.lock().unwrap().insert(ptr as usize, ret_ptr);

            ptr
        }
    };
}

define_api_callback!(get_param_schema, GetParamSchemaArgs);
define_api_callback!(get_handler, GetHandlerArgs);
define_api_callback!(free_handler, FreeHandlerArgs);
define_api_callback!(get_diff, GetDiffArgs);
define_api_callback!(download, DownloadArgs);

unsafe extern "C" fn free(value: APIResultSerialized) {
    STR_REF.lock().unwrap().remove(&(value as usize));
}
