use anydex_plugin_api_lib::{
    common::Handler,
    vector_store::interface::{Chunk, FileId, IndexId, IndexerId, SearchHit, SearchRequest},
};

use crate::plugin::common::DisposableHnandler;

pub trait VectorStore {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(
        &self,
        param: &serde_json::Value,
        index_id: IndexId,
    ) -> Result<DisposableHnandler, String>;

    fn delete_chunks_file(&self, handler: Handler, fild_id: FileId) -> Result<(), String>;
    fn delete_chunks_indexer(&self, handler: Handler, indexer_id: IndexerId) -> Result<(), String>;
    fn clear_chunks(&self, handler: Handler) -> Result<(), String>;
    fn insert_chunks(&self, handler: Handler, chunks: &[Chunk]) -> Result<(), String>;
    fn search(
        &self,
        handler: Handler,
        search_request: &SearchRequest,
    ) -> Result<Vec<SearchHit>, String>;
}
