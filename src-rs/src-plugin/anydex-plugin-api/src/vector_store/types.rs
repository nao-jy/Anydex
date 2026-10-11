use serde::{Deserialize, Serialize};

pub type IndexId = String;
pub type FileId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub id: String,
    pub file_id: FileId,
    pub index_id: IndexId,
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
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub target_file_ids: Vec<FileId>,
    pub offset: i32,
    pub limit: i32,
}
