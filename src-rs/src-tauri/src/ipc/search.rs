use sea_orm::DatabaseConnection;
use tauri::State;
use uuid::Uuid;

use crate::{
    dto::search::{GetFileInfoResult, GrepResult, ListDirectoryResult},
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
#[tracing::instrument(skip(database, uri))]
pub async fn get_file_info(
    index_id: Uuid,
    uri: String,
    database: State<'_, DatabaseConnection>,
) -> Result<GetFileInfoResult, String> {
    usecase::search::get_file_info(index_id, &uri, database.inner()).await
}
