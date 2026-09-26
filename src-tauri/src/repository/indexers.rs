use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, Set, TransactionTrait};
use std::option::Option::None;

use crate::{dto::models::Indexer, repository::models};

pub async fn create_empty_indexer(
    indexer_id: i32,
    database: &DatabaseConnection,
) -> Result<Indexer, DbErr> {
    let transaction = database.begin().await?;

    let model = models::indexer::ActiveModel {
        index_id: Set(indexer_id),
        last_start: Set(None),
        last_success: Set(None),
        last_processed_files: Set(None),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    transaction.commit().await?;
    Ok(Indexer {
        id: model.id,
        index_id: model.index_id,
        last_start: model.last_start,
        last_success: model.last_success,
        last_processed_files: model.last_processed_files,
        file_source: None,
        file_extractor: None,
        schedule: None,
        tasks: Vec::new(),
    })
}
