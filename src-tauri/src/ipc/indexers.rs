use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait};
use tauri::State;

use crate::{dto::models::Schedule, repository};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn create_empty_indexer(
    index_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Indexer, String> {
    let indexer = repository::indexers::create_empty_indexer(index_id, database.inner())
        .await
        .map_err(|error| error.to_string())?;
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
    let transaction = database
        .inner()
        .begin()
        .await
        .map_err(|err| err.to_string())?;

    repository::indexers::delete_indexer(indexer_id, &transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())?;

    tracing::info!(indexer_id = indexer_id, "indexer deleted");
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
    let _model = repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    repository::indexers::upsert_schedule(&database, indexer_id, schedule)
        .await
        .map_err(|err| err.to_string())?;

    tracing::info!(indexer_id = indexer_id, "schedule deleted");
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_schedule(
    indexer_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    repository::models::schedule::Entity::delete_by_indexer_id(indexer_id)
        .exec(database.inner())
        .await
        .map_err(|err| err.to_string())?;

    tracing::info!(indexer_id = indexer_id, "schedule deleted");
    Ok(())
}
