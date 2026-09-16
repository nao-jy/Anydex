use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use tauri::State;

use crate::repository;

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_indexes(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<crate::dto::models::Index>, String> {
    tracing::info!("listing indexes");
    let indexes = repository::indexes::list_indexes(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(count = indexes.len(), "indexes listed");
    Ok(indexes)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn create_empty_index(
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    let index = repository::indexes::create_empty_index(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "empty index created");
    Ok(index)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, name))]
pub async fn rename_index(
    index_id: i32,
    name: String,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    let index = repository::indexes::rename_index(database.inner(), index_id, name)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "index renamed");
    Ok(index)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn set_index_mcp_server(
    index_id: i32,
    enabled: bool,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let model = repository::models::index::Entity::find_by_id(index_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;
    let mut active_model = model.into_active_model();
    active_model.use_mcp_server = Set(enabled);
    active_model.updated_at = Set(Utc::now());
    active_model
        .update(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id, enabled, "MCP index setting updated");
    Ok(())
}
