use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait};

use crate::{dto::models::Schedule, repository};

pub async fn create_empty_indexer(
    index_id: i32,
    database: &DatabaseConnection,
) -> Result<crate::dto::models::Indexer, String> {
    repository::indexers::create_empty_indexer(index_id, database)
        .await
        .map_err(|error| error.to_string())
}

pub async fn delete_indexer(indexer_id: i32, database: &DatabaseConnection) -> Result<(), String> {
    let transaction = database.begin().await.map_err(|err| err.to_string())?;

    repository::indexers::delete_indexer(indexer_id, &transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())
}

pub async fn upsert_schedule(
    indexer_id: i32,
    schedule: Schedule,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    repository::indexers::upsert_schedule(database, indexer_id, schedule)
        .await
        .map_err(|err| err.to_string())
}

pub async fn delete_schedule(indexer_id: i32, database: &DatabaseConnection) -> Result<(), String> {
    repository::models::schedule::Entity::delete_by_indexer_id(indexer_id)
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}
