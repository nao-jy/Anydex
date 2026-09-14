use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::{collections::HashMap, hash::Hash};

// Index information for 
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Index {
    id: i32,
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    file_source_ids: Vec<FileSource>,
}

// Use OpenDAL to access files from different sources
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FileSource {
    id: i32,
    name: String,
    uri: String,
    oauth_id: Option<i32>,
    api_key: Option<String>,
    extras: Option<HashMap<String, String>>,
}

// OAuth without client secret (PKCE flow)
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OAuth {
    id: i32,
    name: String,
    client_id: String,
    authorization_endpoint: String,
    token_endpoint: String,
    refresh_endpoint: String,
    scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OAuthAuthentication {
    oauth_id: i32,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    index_id: i32,
    file_source_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub enum Modality{
    Text, Image, Other(String)
}
impl PartialEq for Modality {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // 両方がActiveで、中身の文字列が同じならtrue
            (Modality::Text, Modality::Text) => true,
            (Modality::Image, Modality::Image) => true,
            (Modality::Other(a), Modality::Other(b)) => a == b,
            
            // それ以外（ActiveとInactiveの比較など）はすべてfalse
            _ => false,
        }
    }
}
impl Eq for Modality {}
impl Hash for Modality {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Modality::Text => {
                state.write_u8(0);
            }
            Modality::Image => {
                state.write_u8(1);
            }
            Modality::Other(s) => {
                state.write_u8(2);
                s.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APISchema {
    endpoint: String,
    request_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vectorizer {
    id: i32,
    name: String,
    endpoints: HashMap<Modality, APISchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Chunk {
    id: String,
    index_id: i32,
    source_id: i32,
    file_path: String,
    content: Option<String>,
    content_vector: Option<Vec<f32>>,
    modality: Modality,
    metadata: Option<HashMap<String, String>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}