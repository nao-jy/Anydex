use sea_orm::DatabaseConnection;
use std::path::Path;
use tauri::State;
use uuid::Uuid;

use crate::{
    dto::search::{
        GetFileInfoResult, GrepResult, ListDirectoryResult, SemanticSearchQuery,
        SemanticSearchResult,
    },
    usecase,
};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_directory(
    index_id: Uuid,
    path: String,
    database: State<'_, DatabaseConnection>,
) -> Result<ListDirectoryResult, String> {
    usecase::search::list_directory(index_id, &path, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, pattern, target))]
pub async fn grep(
    index_id: Uuid,
    pattern: String,
    target: String,
    database: State<'_, DatabaseConnection>,
) -> Result<GrepResult, String> {
    usecase::search::grep(index_id, &pattern, &target, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, query))]
pub async fn semantic_search(
    index_id: Uuid,
    query: SemanticSearchQuery,
    target: String,
    database: State<'_, DatabaseConnection>,
) -> Result<SemanticSearchResult, String> {
    usecase::search::semantic_search(index_id, query, &target, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, uri))]
pub async fn get_file_info(
    index_id: Uuid,
    uri: String,
    database: State<'_, DatabaseConnection>,
) -> Result<GetFileInfoResult, String> {
    usecase::search::get_file_info(index_id, &uri, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, uri, destination))]
pub async fn download_file(
    index_id: Uuid,
    uri: String,
    destination: String,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::search::get_file_info(index_id, &uri, database.inner()).await?;
    let downloaded_path = usecase::files::download_file(index_id, &uri, database.inner()).await?;
    std::fs::copy(downloaded_path, Path::new(&destination))
        .map_err(|err| format!("failed to save downloaded file to '{destination}': {err}"))?;
    Ok(())
}
