use std::time::Duration;

use uuid::Uuid;

pub async fn list_directory(index_id: Uuid, path: &str) -> Result<Vec<String>, String> {
    Ok(vec![])
}

pub async fn grep(index_id: Uuid, pattern: &str, target: &str, semantic: bool) {}

pub async fn get_file_info(index_id: Uuid, uri: &str) {}
