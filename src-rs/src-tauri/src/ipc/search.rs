use crate::{dto::models::SearchResult, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(query), fields(query_length))]
pub async fn search_mock(query: String) -> Vec<SearchResult> {
    usecase::search::search_mock(query).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument]
pub fn greet(name: &str) -> String {
    usecase::search::greet(name)
}
