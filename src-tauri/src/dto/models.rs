use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Index configuration returned to the UI.
///
/// This mirrors the aggregate rooted at `repository::models::index`. Related
/// entities are loaded explicitly by the repository before constructing it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub use_mcp_server: bool,
    pub embedder: Embedder,
    pub vector_store: VectorStore,
    pub indexers: Vec<Indexer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Embedder {
    pub id: i32,
    pub index_id: i32,
    pub name: String,
    pub dimensions: i32,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VectorStore {
    pub id: i32,
    pub index_id: i32,
    pub name: String,
    pub dimensions: i32,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Indexer {
    pub id: i32,
    pub index_id: i32,
    pub extractor_name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
    pub last_start: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_processed_files: Option<i32>,
    pub file_source: FileSource,
    pub schedule: Schedule,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: i32,
    pub indexer_id: i32,
    pub timezone: String,
    pub dtstart: DateTime<Utc>,
    pub rfc5545_rule: String,
    pub misfire_policy: String,
}

/// OpenDAL file-source configuration and the files discovered from it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSource {
    pub id: i32,
    pub indexer_id: i32,
    pub name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
    pub files: Vec<File>,
    pub oauth_client: Option<OAuthClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: i32,
    pub file_source_id: i32,
    pub name: String,
    pub uri: String,
    pub content_type: String,
    /// Byte count serialized as a decimal string to avoid JavaScript integer
    /// precision loss. Convert from the repository model with `to_string()`.
    pub size: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i32,
    pub indexer_id: i32,
    pub task_type: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    #[specta(type = specta_typescript::Any)]
    pub state: Value,
}

/// OAuth client configuration for management screens. PKCE has no client
/// secret, but token values are never included here.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClient {
    pub id: i32,
    pub name: String,
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub refresh_endpoint: Option<String>,
    #[specta(type = specta_typescript::Any)]
    pub scopes: Value,
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
