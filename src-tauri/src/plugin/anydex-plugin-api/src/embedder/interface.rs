use crate::common::CSlice;
use std::ffi::{c_char, c_void};

pub type Handler = u64;

/// ABI version implemented by this host.
pub const DYNAMIC_EMBEDDER_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
pub struct DynamicEmbedderV1 {
    /// Must equal [`DYNAMIC_EMBEDDER_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema: unsafe extern "C" fn() -> *const c_char,
    pub get_handler: unsafe extern "C" fn(params: *const c_char) -> Handler,
    pub free_handler: unsafe extern "C" fn(handler: Handler),
    pub embed:
        unsafe extern "C" fn(handler: Handler, file_path: *const c_char) -> *const CSlice<f32>,
    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: *const c_void),
}

pub enum DynamicEmbedder {
    V1(DynamicEmbedderV1),
}
