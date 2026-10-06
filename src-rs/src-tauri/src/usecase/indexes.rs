use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set, TransactionTrait,
};

use crate::{
    dto::models::Index,
    plugin::{entry::VECTOR_STORE_MGR, vector_store::interface::VectorStore},
    repository,
};

pub async fn list_indexes(database: &DatabaseConnection) -> Result<Vec<Index>, String> {
    tracing::info!("listing indexes");
    let indexes = repository::indexes::list_indexes(database)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(count = indexes.len(), "indexes listed");
    Ok(indexes)
}

pub async fn create_empty_index(database: &DatabaseConnection) -> Result<Index, String> {
    repository::indexes::create_empty_index(database)
        .await
        .map_err(|error| error.to_string())
}

pub async fn rename_index(
    index_id: i32,
    name: String,
    database: &DatabaseConnection,
) -> Result<Index, String> {
    let index = repository::indexes::rename_index(database, index_id, name)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "index renamed");
    Ok(index)
}

pub async fn put_description(
    index_id: i32,
    description: String,
    database: &DatabaseConnection,
) -> Result<Index, String> {
    let index = repository::indexes::put_description(database, index_id, description)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "index description updated");
    Ok(index)
}

pub async fn set_index_mcp_server(
    index_id: i32,
    enabled: bool,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let model = repository::models::index::Entity::find_by_id(index_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;
    let mut active_model = model.into_active_model();
    active_model.use_mcp_server = Set(enabled);
    active_model.updated_at = Set(Utc::now());
    active_model
        .update(database)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id, enabled, "MCP index setting updated");
    Ok(())
}

pub async fn delete_index(index_id: i32, database: &DatabaseConnection) -> Result<(), String> {
    let transaction = database.begin().await.map_err(|err| err.to_string())?;

    repository::models::embedder::Entity::delete_by_index_id(index_id)
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let vector_store = repository::models::vector_store::Entity::find_by_index_id(index_id)
        .one(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    repository::models::vector_store::Entity::delete_by_index_id(index_id)
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

    repository::models::indexer::Entity::delete_many()
        .filter(repository::models::indexer::Column::IndexId.eq(index_id))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    repository::models::index::Entity::delete_by_id(index_id)
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())?;

    if let Some(vector_store) = vector_store {
        let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
        let mgr = mgr_mtx.as_mut().unwrap();
        if let Some(plugin) = mgr.get_plugin(&vector_store.name) {
            let handler = plugin
                .get_handler(&vector_store.params, index_id)
                .map_err(|err| err.to_string())?;
            plugin
                .clear_chunks(handler.handler)
                .map_err(|err| err.to_string())?;
        }
    }

    tracing::info!(index_id, "index deleted");
    Ok(())
}
