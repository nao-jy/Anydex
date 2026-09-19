use std::ffi::c_char;

use libloading::{Library, Symbol};

use crate::plugin::common::{host_log, LogLevel};

// struct header part
// plugin must start with this header
#[repr(C)]
pub struct DynamicPluginHeader {
    pub abi_version: i32,
    pub struct_size: usize,
}

#[repr(C)]
pub struct PluginInit {
    pub abi_version: i32,
    pub struct_size: usize,
    pub log: unsafe extern "C" fn(log_level: LogLevel, message: *const c_char),
}

// Plugin entrypoint: `get_plugin`.
pub type GetPlugin<T> = unsafe extern "C" fn(*const PluginInit) -> *const T;

// Get plugin pointer
pub fn get_plugin<T>(library: &Library) -> Result<*const T, String> {
    let get_plugin: Symbol<GetPlugin<T>> = unsafe {
        library
            .get(b"get_plugin")
            .map_err(|error| format!("missing get_plugin: {error}"))?
    };
    let init = PluginInit {
        abi_version: 1,
        struct_size: std::mem::size_of::<PluginInit>(),
        log: host_log,
    };
    let plugin = unsafe { get_plugin(&init) };
    if plugin.is_null() {
        return Err("get_plugin returned a null pointer".to_string());
    }
    Ok(plugin)
}
