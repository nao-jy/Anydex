use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFileInfo {
    pub file_uri: String,
    pub size: usize,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputFileInfo {
    pub file_uri: String,
    pub file_path: String,
    pub file_name: String,
    pub file_dir: String,
    pub content_type: String,
    pub size: usize,
    pub updated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FileIterationInfo {
    AddedOrUpdated(OutputFileInfo),
    Deleted(String),
}
