use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type IndexId = i32;
pub type IndexerId = i32;
pub type FileId = i32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub chunk: Chunk,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterCondition {
    pub dirs: Vec<String>,
    pub content_type: Vec<String>,
    pub file_name: Vec<String>,
    pub file_ext: Vec<String>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub embedding: Vec<f32>,
    pub filter_condition: FilterCondition,
    pub offset: i32,
    pub limit: i32,
}
