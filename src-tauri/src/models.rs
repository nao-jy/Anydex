use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::{collections::HashMap};

// A searchable index and the sources that are synchronized into it.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub vectorizer: Vectorizer,
    pub indexer: Vec<Indexer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub timezone: String,        // "Asia/Tokyo"
    pub dtstart: DateTime<Utc>,  // 繰り返しの基準時刻
    pub rule: String,            // RFC 5545 RRULE
    pub misfire_policy: String, // "skip" or "catchup"
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Indexer {
    pub file_source: FileSource,
    pub schedule: Schedule,
    pub last_start: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_processed_files: Option<i32>,
}

// Use OpenDAL to access files from different sources
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileSource {
    pub name: String,
    pub uri: String,
    pub extras: HashMap<String, String>,
}

/// A semantic-search hit returned to the UI.
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
pub struct APISchema {
    pub endpoint: String,
    pub request_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Vectorizer {
    pub name: String,
    pub endpoints: HashMap<String, APISchema>,
    pub dimensions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Chunk {
    pub id: String,
    pub index_id: i32,
    pub source_id: i32,
    pub file_path: String,
    pub content_type: String,
    pub content: Option<String>,
    pub content_vector: Option<Vec<f32>>,
    pub modality: String,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
