use sea_orm::DatabaseConnection;
use tauri::State;

use crate::usecase;

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_indexes(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<crate::dto::models::Index>, String> {
    usecase::indexes::list_indexes(database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn create_empty_index(
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    usecase::indexes::create_empty_index(database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, name))]
pub async fn rename_index(
    index_id: i32,
    name: String,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    usecase::indexes::rename_index(index_id, name, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, description))]
pub async fn put_description(
    index_id: i32,
    description: String,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    usecase::indexes::put_description(index_id, description, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn set_index_mcp_server(
    index_id: i32,
    enabled: bool,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexes::set_index_mcp_server(index_id, enabled, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_index(
    index_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexes::delete_index(index_id, database.inner()).await
}
