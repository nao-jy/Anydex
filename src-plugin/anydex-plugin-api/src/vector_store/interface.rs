use crate::common::{CResult, CSlice, Handler};
use std::ffi::{c_char, c_void};

pub const INVALID_HANDLE: u64 = u64::MAX;
pub type IndexId = i32;
pub type IndexerId = i32;
pub type FileId = i32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CChunk {
    pub id: i32,
    pub index_id: IndexId,
    pub indexer_id: IndexerId,
    pub file_id: FileId,
    pub uri: *const c_char,
    pub content_type: *const c_char,
    pub content: *const c_char,
    pub content_vector: *const CSlice<f32>,
    pub metadata: *const c_char,
    pub created_at_utc: i64,
    pub updated_at_utc: i64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CSearchHit {
    pub chunk: CChunk,
    pub score: f32,
}
#[repr(C)]
pub struct CSearchRequest {
    pub embedding: CSlice<f32>,
    pub additional_cond_sql: *const c_char,
    pub offset: i32,
    pub limit: i32,
}

/// ABI version implemented by this host.
pub const DYNAMIC_VECTOR_STORE_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicVectorStoreV1 {
    /// Must equal [`DYNAMIC_VECTOR_STORE_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema: unsafe extern "C" fn() -> *const c_char,
    pub get_handler: unsafe extern "C" fn(params: *const c_char, index_id: IndexId) -> Handler,
    pub free_handler: unsafe extern "C" fn(handler: Handler),

    pub delete_chunk_file: unsafe extern "C" fn(handler: Handler, fild_id: FileId),
    pub delete_chunk_indexer: unsafe extern "C" fn(handler: Handler, indexer_id: IndexerId),
    pub insert_chunk: unsafe extern "C" fn(handler: Handler, chunk: CChunk),
    pub clear_chunk: unsafe extern "C" fn(handler: Handler),
    pub search: unsafe extern "C" fn(
        handler: Handler,
        search_request: CSearchRequest,
    ) -> *const CResult<CSlice<CSearchHit>>,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: *const c_void),
}

pub enum DynamicVectorStore {
    V1(DynamicVectorStoreV1),
}
