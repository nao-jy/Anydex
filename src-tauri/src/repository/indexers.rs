use crate::plugin::vector_store::interface::VectorStore;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter, Set, TransactionTrait,
};
use std::option::Option::None;

use crate::{
    dto::models::{Indexer, Schedule},
    plugin::entry::VECTOR_STORE_MGR,
    repository::models,
};

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
    let indexer = models::indexer::Entity::find_by_id(indexer_id)
        .one(transaction)
        .await?;

    let vector_store = match &indexer {
        Some(indexer_some) => {
            models::vector_store::Entity::find_by_index_id(indexer_some.index_id)
                .one(transaction)
                .await?
        }
        None => None,
    };

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

    if let Some(vector_store_some) = &vector_store {
        if let Some(indexer_some) = &indexer {
            let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
            let mgr = mgr_mtx.as_mut().unwrap();
            let plugin = mgr.get_plugin(&vector_store_some.name);
            if let Some(plugin_some) = plugin {
                let handler = plugin_some
                    .get_handler(&vector_store_some.params, indexer_some.index_id)
                    .map_err(|err| DbErr::Custom(err.to_string()))?;
                let _ = plugin_some
                    .delete_chunks_indexer(handler.handler, indexer_some.id)
                    .map_err(|err| DbErr::Custom(err.to_string()));
            }
        }
    }

    Ok(())
}

pub async fn upsert_schedule(
    database: &DatabaseConnection,
    indexer_id: i32,
    schedule: Schedule,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let emb_model = models::schedule::Entity::find_by_indexer_id(indexer_id);
    let emb = emb_model.one(&transaction).await?;

    if let Some(emb_ent) = emb {
        let mut active_model = emb_ent.into_active_model();
        active_model.rfc5545_rrule = Set(schedule.rfc5545_rrule);
        active_model.catchup_when_missed = Set(schedule.catchup_when_missed);

        active_model.update(&transaction).await?;
    } else {
        models::schedule::ActiveModel {
            indexer_id: Set(schedule.indexer_id),
            rfc5545_rrule: Set(schedule.rfc5545_rrule),
            catchup_when_missed: Set(schedule.catchup_when_missed),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}
