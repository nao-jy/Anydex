use sea_orm::DatabaseConnection;
use tauri::State;

use crate::{dto::models::Schedule, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn create_empty_indexer(
    index_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Indexer, String> {
    let indexer = usecase::indexers::create_empty_indexer(index_id, database.inner()).await?;
    tracing::info!(indexer_id = indexer.id, "empty indexer created");
    Ok(indexer)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_indexer(
    indexer_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexers::delete_indexer(indexer_id, database.inner()).await?;
    tracing::info!(indexer_id, "indexer deleted");
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_schedule(
    indexer_id: i32,
    schedule: Schedule,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexers::upsert_schedule(indexer_id, schedule, database.inner()).await?;
    tracing::info!(indexer_id, "schedule upserted");
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_schedule(
    indexer_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::indexers::delete_schedule(indexer_id, database.inner()).await?;
    tracing::info!(indexer_id, "schedule deleted");
    Ok(())
}
