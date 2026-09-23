use crate::common::{CResult, CSlice};
use std::ffi::{c_char, c_void};

pub const INVALID_HANDLE: u64 = u64::MAX;
pub type Handler = u64;
pub type CFileId = *const c_char;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CInputFileInfo {
    pub file_id: CFileId,
    pub size: usize,
    pub updated_at_utc: i64,
    pub file_hash: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct COutputFileInfo {
    pub file_id: CFileId,
    pub name: *const c_char,
    // downloaded file path
    pub file_path: *const c_char,
    pub content_type: *const c_char,
    pub size: usize,
    pub updated_at_utc: i64,
    pub metadata: *const c_char,
    pub file_hash: u64,
}

pub struct CFileIterationInfo {
    pub added_or_updated: *const COutputFileInfo,
    pub deleted: CFileId,
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

    pub next: unsafe extern "C" fn(handler: Handler) -> *const CResult<CFileIterationInfo>,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: *const c_void),
}

pub enum DynamicFileSource {
    V1(DynamicFileSourceV1),
}
