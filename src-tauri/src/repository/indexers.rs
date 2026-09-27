use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};
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

pub async fn delete_indexer(
    indexer_id: i32,
    transaction: &DatabaseTransaction,
) -> Result<(), DbErr> {
    let _result = models::file_extractor::Entity::delete_by_indexer_id(indexer_id)
        .exec(transaction)
        .await?;

    let file_srcs = models::file_source::Entity::find_by_indexer_id(indexer_id)
        .all(transaction)
        .await?;

    for file_src in file_srcs {
        let _result = models::oauth_authentication::Entity::delete_by_id(file_src.id)
            .exec(transaction)
            .await?;

        let _result = models::file::Entity::delete_many()
            .filter(models::file::Column::FileSourceId.eq(file_src.id))
            .exec(transaction)
            .await?;
    }

    let _result = models::file_source::Entity::delete_by_indexer_id(indexer_id)
        .exec(transaction)
        .await?;

    let _result = models::schedule::Entity::delete_by_indexer_id(indexer_id)
        .exec(transaction)
        .await?;

    let _result = models::task::Entity::delete_many()
        .filter(models::task::Column::IndexerId.eq(indexer_id))
        .exec(transaction)
        .await?;

    let _result = models::indexer::Entity::delete_by_id(indexer_id)
        .exec(transaction)
        .await?;

    Ok(())
}
