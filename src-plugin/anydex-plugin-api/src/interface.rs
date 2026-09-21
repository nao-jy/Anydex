use crate::common::LogLevel;
use std::ffi::c_char;

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
