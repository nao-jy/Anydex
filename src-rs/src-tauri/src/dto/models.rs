use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::plugin::entry::PluginType;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: i32,
    pub file_source_id: i32,
    pub name: String,
    pub file_uri: String,
    pub file_path: String,
    pub file_dir: String,
    pub file_name: String,
    pub content_type: String,
    /// Byte count serialized as a decimal string to avoid JavaScript integer
    /// precision loss. Convert from the repository model with `to_string()`.
    pub size: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Serialized as a decimal string to avoid JavaScript integer precision loss.
    pub id: String,
    pub index_id: String,
    pub job_type: String,
    pub status: String,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct IndexingHistory {
    pub id: u64,
    pub index_id: String,
    pub started_at: DateTime<Utc>,
}

/// A semantic-search hit returned to the UI. This is not a persistence entity.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub source: String,
    pub content_type: String,
    pub score: String,
    pub excerpt: String,
    pub updated_at: String,
}

/// Status of the local MCP server exposed by the application.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub port: u16,
    pub started_at: DateTime<Utc>,
}

/// Locations and plugin discovery settings displayed by the settings screen.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSettings {
    pub database_path: String,
    pub log_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub plugin_type: PluginType,
    pub path: String,
    pub plugins: Vec<String>,
}
