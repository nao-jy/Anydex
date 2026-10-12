use sea_orm::DatabaseConnection;
use tauri::State;
use uuid::Uuid;

use crate::{
    repository::json::models::{Index, Schedule},
    usecase,
};

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn upsert_schedule(index_id: Uuid, schedule: Option<Schedule>) -> Result<(), String> {
    usecase::indexes::upsert_schedule(index_id, schedule).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn list_indexes() -> Result<Vec<Index>, String> {
    usecase::indexes::list_indexes().await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn create_empty_index() -> Result<(), String> {
    usecase::indexes::create_empty_index().await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(name))]
pub async fn rename_index(index_id: Uuid, name: String) -> Result<(), String> {
    usecase::indexes::rename_index(index_id, name).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(description))]
pub async fn put_description(index_id: Uuid, description: String) -> Result<(), String> {
    usecase::indexes::put_description(index_id, description).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn set_index_mcp_server(index_id: Uuid, enabled: bool) -> Result<(), String> {
    usecase::indexes::set_index_mcp_server(index_id, enabled).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_index(
    index_id: Uuid,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexes::delete_index(index_id, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn start_indexing(
    index_id: Uuid,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexes::start_indexing(index_id, database.inner()).await
}
