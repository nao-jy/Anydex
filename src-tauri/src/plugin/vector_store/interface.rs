use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};

use libloading::Library;

use crate::plugin::manager::LibraryLoader;

type Handler = u64;
type IndexId = i32;
type IndexerId = i32;
type FileId = i32;

pub struct Chunk {
    pub uuid: String,
    pub index_id: IndexId,
    pub indexer_id: IndexerId,
    pub file_id: FileId,
    pub uri: String,
    pub content_type: String,
    pub content: Option<String>,
    pub content_vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
}

pub struct SearchRequest {
    pub embedding: Vec<f32>,
    pub limit: usize,
    pub min_score: Option<f32>,
    pub filter: ChunkFilter,
}

pub struct ChunkFilter {
    pub file_ids: Option<Vec<FileId>>,
    pub content_types: Option<Vec<String>>,
}

pub trait VectorStore {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(&self, param: &serde_json::Value, index_id: IndexId) -> Handler;
    fn free_handler(&self, handler: Handler);

    fn delete_chunk(&self, handler: Handler, fild_id: FileId);
    fn insert_chunk(&self, handler: Handler, chunk: &Chunk);
    fn query(&self, search_request: &SearchRequest) -> Vec<SearchHit>;
}
