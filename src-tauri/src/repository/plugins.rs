use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DbErr, IntoActiveModel, Set, TransactionTrait,
};
use std::option::Option::Some;

use crate::repository::models;

pub async fn upsert_embedder(
    database: &DatabaseConnection,
    index_id: i32,
    embedder_name: String,
    params: serde_json::Value,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let emb_model = models::embedder::Entity::find_by_index_id(index_id);
    let emb = emb_model.one(&transaction).await?;

    if let Some(emb_ent) = emb {
        let mut active_model = emb_ent.into_active_model();
        active_model.name = Set(embedder_name);
        active_model.params = Set(params);

        active_model.update(&transaction).await?;
    } else {
        models::embedder::ActiveModel {
            name: Set(embedder_name),
            index_id: Set(index_id),
            params: Set(params),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn upsert_vector_store(
    database: &DatabaseConnection,
    index_id: i32,
    vector_store_name: String,
    params: serde_json::Value,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let vec_model = models::vector_store::Entity::find_by_index_id(index_id);
    let vec = vec_model.one(&transaction).await?;

    if let Some(vec_ent) = vec {
        let mut active_model = vec_ent.into_active_model();
        active_model.name = Set(vector_store_name);
        active_model.params = Set(params);

        active_model.update(&transaction).await?;
    } else {
        models::vector_store::ActiveModel {
            name: Set(vector_store_name),
            index_id: Set(index_id),
            params: Set(params),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn upsert_file_extractor(
    database: &DatabaseConnection,
    indexer_id: i32,
    file_extractor_name: String,
    params: serde_json::Value,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let emb_model = models::file_extractor::Entity::find_by_indexer_id(indexer_id);
    let emb = emb_model.one(&transaction).await?;

    if let Some(emb_ent) = emb {
        let mut active_model = emb_ent.into_active_model();
        active_model.name = Set(file_extractor_name);
        active_model.params = Set(params);

        active_model.update(&transaction).await?;
    } else {
        models::file_extractor::ActiveModel {
            name: Set(file_extractor_name),
            indexer_id: Set(indexer_id),
            params: Set(params),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn upsert_file_source(
    database: &DatabaseConnection,
    indexer_id: i32,
    file_source_name: String,
    params: serde_json::Value,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let vec_model = models::file_source::Entity::find_by_indexer_id(indexer_id);
    let vec = vec_model.one(&transaction).await?;

    if let Some(vec_ent) = vec {
        let mut active_model = vec_ent.into_active_model();
        active_model.name = Set(file_source_name);
        active_model.params = Set(params);

        active_model.update(&transaction).await?;
    } else {
        models::file_source::ActiveModel {
            name: Set(file_source_name),
            indexer_id: Set(indexer_id),
            params: Set(params),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}
