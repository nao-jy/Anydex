use anydex_plugin_api_lib::vector_store::interface::{FileId, Handler, IndexId, IndexerId};
use chrono::{DateTime, Utc};

pub struct Chunk {
    pub id: i32,
    pub index_id: IndexId,
    pub indexer_id: IndexerId,
    pub file_id: FileId,
    pub uri: String,
    pub content_type: String,
    pub content: Option<String>,
    pub content_vector: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
}

pub struct SearchRequest {
    pub embedding: Vec<f32>,
    pub additional_cond_sql: String,
    pub limit: i32,
    pub offset: i32,
}

pub trait VectorStore {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(&self, param: &serde_json::Value, index_id: IndexId) -> Result<Handler, String>;
    fn free_handler(&self, handler: Handler) -> Result<(), String>;

    fn delete_chunk_file(&self, handler: Handler, fild_id: FileId) -> Result<(), String>;
    fn delete_chunk_indexer(&self, handler: Handler, indexer_id: IndexerId) -> Result<(), String>;
    fn clear_chunk(&self, handler: Handler) -> Result<(), String>;
    fn insert_chunk(&self, handler: Handler, chunk: &Chunk) -> Result<(), String>;
    fn search(
        &self,
        handler: Handler,
        search_request: &SearchRequest,
    ) -> Result<Vec<SearchHit>, String>;
}
