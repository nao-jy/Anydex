use crate::common::{CPair, CResult, CSlice};
use std::ffi::{c_char, c_void};

pub type Handler = u64;
pub type CFileId = *const c_char;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CInputFileInfo {
    pub file_id: CFileId,
    pub size: usize,
    pub updated_at_utc: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct COutputFileInfo {
    pub file_id: CFileId,
    pub name: *const c_char,
    pub file_path: *const c_char,
    pub content_type: *const c_char,
    pub size: usize,
    pub updated_at_utc: i64,
    pub metadata: CSlice<CPair<*const c_char, *const c_char>>,
}

/// ABI version implemented by this host.
pub const DYNAMIC_FILE_SOURCE_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicFileSourceV1 {
    /// Must equal [`DYNAMIC_FILE_SOURCE_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema: unsafe extern "C" fn() -> *const c_char,
    pub get_handler: unsafe extern "C" fn(
        params: *const c_char,
        last_indexed_files: CSlice<CInputFileInfo>,
        download_dir: *const c_char,
    ) -> Handler,
    pub free_handler: unsafe extern "C" fn(handler: Handler),

    pub next_added_or_changed:
        unsafe extern "C" fn(handler: Handler) -> *const CResult<*const COutputFileInfo>,
    pub next_deleted: unsafe extern "C" fn(handler: Handler) -> *const CResult<CFileId>,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: *const c_void),
}

pub enum DynamicFileSource {
    V1(DynamicFileSourceV1),
}
