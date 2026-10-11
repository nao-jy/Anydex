use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GrepResult {
    pub matches: Vec<GrepMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GrepMatch {
    pub file: FileMetadata,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoryResult {
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct GetFileInfoResult {
    pub file: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    /// Serialized as a decimal string to avoid JavaScript integer precision loss.
    pub id: String,
    pub index_id: String,
    pub name: String,
    pub file_uri: String,
    pub file_path: String,
    pub file_dir: String,
    pub file_name: String,
    pub content_type: String,
    /// A decimal string avoids losing precision when represented by JavaScript.
    pub size: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub file_hash: Option<String>,
}
