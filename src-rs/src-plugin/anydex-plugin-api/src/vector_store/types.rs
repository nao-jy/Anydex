use serde::{Deserialize, Serialize};

pub type IndexId = i32;
pub type IndexerId = i32;
pub type FileId = i32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub id: i32,
    pub file_id: FileId,
    pub indexer_id: IndexerId,
    pub content: String,
    pub content_vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub embedding: Vec<f32>,
    pub target_file_ids: Vec<FileId>,
    pub offset: i32,
    pub limit: i32,
}
