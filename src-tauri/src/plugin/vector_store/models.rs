
pub struct Chunk {
    pub id: String,
    pub index_id: i32,
    pub file_id: i32,
    pub file_path: String,
    pub content_type: String,
    pub content: Option<String>,
    pub content_vector: Option<Vec<f32>>,
    pub modality: String,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub extras: Option<HashMap<String, String>>,
}
