use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Embedder {
    pub name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VectorStore {
    pub name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub rfc5545_rrule: String,
    pub catchup_when_missed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileExtractor {
    pub name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSource {
    pub name: String,
    #[specta(type = specta_typescript::Any)]
    pub params: Value,
    pub oauth_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub use_mcp_server: bool,
    pub file_source: Option<FileSource>,
    pub file_extractor: Option<FileExtractor>,
    pub embedder: Option<Embedder>,
    pub vector_store: Option<VectorStore>,
    pub schedule: Option<Schedule>,
}
impl Default for Index {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "New Index".to_string(),
            description: "".to_string(),
            use_mcp_server: true,
            file_source: None,
            file_extractor: None,
            embedder: None,
            vector_store: None,
            schedule: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct OAuthClient {
    pub id: Uuid,
    pub name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub refresh_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub scopes: Vec<String>,
}
impl Default for OAuthClient {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "New OAuth".to_string(),
            client_id: "".to_string(),
            client_secret: None,
            authorization_endpoint: "".to_string(),
            token_endpoint: "".to_string(),
            refresh_endpoint: None,
            revocation_endpoint: None,
            scopes: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub indexes: Vec<Index>,
    pub oauth_clients: Vec<OAuthClient>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            indexes: [].to_vec(),
            oauth_clients: [].to_vec(),
        }
    }
}
