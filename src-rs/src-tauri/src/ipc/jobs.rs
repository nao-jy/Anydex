use sea_orm::DatabaseConnection;
use tauri::State;
use uuid::Uuid;

use crate::usecase;

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_active_jobs(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<crate::dto::models::Job>, String> {
    usecase::jobs::list_active_jobs(database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn start_indexing(
    database: State<'_, DatabaseConnection>,
    indexer_id: Uuid,
) -> Result<(), String> {
    usecase::jobs::start_indexing(database.inner(), indexer_id).await
}
