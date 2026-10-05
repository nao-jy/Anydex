use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::APIResultSerialized;
use std::ffi::c_char;

pub type IndexId = i32;
pub type IndexerId = i32;
pub type FileId = i32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub id: i32,
    pub index_id: IndexId,
    pub indexer_id: IndexerId,
    pub file_id: FileId,
    pub uri: String,
    pub file_name: String,
    pub file_dir: String,
    pub content_type: String,
    pub content: String,
    pub content_vector: Vec<f32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCondition {
    pub dirs: Vec<String>,
    pub content_type: Vec<String>,
    pub file_name: Vec<String>,
    pub file_ext: Vec<String>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchRequest {
    pub embedding: Vec<f32>,
    pub filter_condition: FilterCondition,
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
    pub get_param_schema:
        unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub get_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub free_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    pub delete_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub insert_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub clear_chunks: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub search: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: APIResultSerialized),
}

pub enum DynamicVectorStore {
    V1(DynamicVectorStoreV1),
}
