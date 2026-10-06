use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set, TransactionTrait,
};
use tauri::State;

use crate::{
    plugin::{entry::VECTOR_STORE_MGR, vector_store::interface::VectorStore},
    repository,
};

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
#[tracing::instrument(skip(database, description))]
pub async fn put_description(
    index_id: i32,
    description: String,
    database: State<'_, DatabaseConnection>,
) -> Result<crate::dto::models::Index, String> {
    let index = repository::indexes::put_description(database.inner(), index_id, description)
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

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn delete_index(
    index_id: i32,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let transaction = database
        .inner()
        .begin()
        .await
        .map_err(|err| err.to_string())?;

    let _result = repository::models::embedder::Entity::delete_by_index_id(index_id)
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let vector_store = repository::models::vector_store::Entity::find_by_index_id(index_id)
        .one(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let _result = repository::models::vector_store::Entity::delete_by_index_id(index_id)
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let indexers = repository::models::indexer::Entity::find()
        .filter(repository::models::indexer::Column::IndexId.eq(index_id))
        .all(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    for indexer in indexers {
        repository::indexers::delete_indexer(indexer.id, &transaction)
            .await
            .map_err(|err| err.to_string())?;
    }

    let _result = repository::models::indexer::Entity::delete_many()
        .filter(repository::models::indexer::Column::IndexId.eq(index_id))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let _result = repository::models::index::Entity::delete_by_id(index_id)
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())?;

    // delete vector chunks
    match vector_store {
        Some(vector_store_some) => {
            let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
            let mgr = mgr_mtx.as_mut().unwrap();
            let plugin = mgr.get_plugin(&vector_store_some.name);
            if let Some(plugin_some) = plugin {
                let handler = plugin_some.get_handler(&vector_store_some.params, index_id)?;
                plugin_some.clear_chunks(handler.handler)?
            }
        }
        None => {}
    }

    tracing::info!(index_id = index_id, "index deleted");
    Ok(())
}
