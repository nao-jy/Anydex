//! Persistence operations for the index aggregate.

use std::collections::HashSet;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::{
    dto::models::{
        Embedder, File, FileSource, Index, Indexer, OAuthClient, Schedule, Task, VectorStore,
    },
    repository::models,
};

/// Creates an index before any source or search configuration has been added.
///
/// The title is made unique so repeatedly clicking the create action always
/// creates a distinct record. Configuration relations deliberately remain
/// absent: they are filled in by their respective setup flows.
pub async fn create_empty_index(database: &DatabaseConnection) -> Result<Index, DbErr> {
    let transaction = database.begin().await?;
    let name = next_empty_index_name(&transaction).await?;
    let now = Utc::now();

    let model = models::index::ActiveModel {
        name: Set(name),
        description: Set(String::new()),
        created_at: Set(now),
        updated_at: Set(now),
        use_mcp_server: Set(false),
        ..Default::default()
    }
    .insert(&transaction)
    .await?;

    transaction.commit().await?;
    get_index(database, model.id).await
}

/// Renames an index and records when its configuration was last changed.
pub async fn rename_index(
    database: &DatabaseConnection,
    index_id: i32,
    name: String,
) -> Result<Index, DbErr> {
    let name = name.trim();
    if name.is_empty() {
        return Err(DbErr::Custom("index name cannot be empty".into()));
    }

    let model = models::index::Entity::find_by_id(index_id)
        .one(database)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("index {index_id} was not found")))?;
    if model.name == name {
        return load_index(database, model).await;
    }

    let mut active_model = model.into_active_model();
    active_model.name = Set(name.to_owned());
    active_model.updated_at = Set(Utc::now());
    active_model.update(database).await?;

    get_index(database, index_id).await
}

/// Loads every index aggregate from the application database.
pub async fn list_indexes(database: &DatabaseConnection) -> Result<Vec<Index>, DbErr> {
    let models = models::index::Entity::find()
        .order_by_asc(models::index::Column::Id)
        .all(database)
        .await?;

    let mut indexes = Vec::with_capacity(models.len());
    for model in models {
        indexes.push(load_index(database, model).await?);
    }
    Ok(indexes)
}

pub async fn upsert_embedder(
    database: &DatabaseConnection,
    index_id: i32,
    embedder_name: String,
    dimensions: i32,
    params: serde_json::Value,
) -> Result<(), DbErr> {
    let transaction = database.begin().await?;

    let emb_model = models::embedder::Entity::find_by_index_id(index_id);
    let emb = emb_model.one(&transaction).await?;

    if let Some(emb_ent) = emb {
        let mut active_model = emb_ent.into_active_model();
        active_model.name = Set(embedder_name);
        active_model.dimensions = Set(dimensions);
        active_model.params = Set(params);

        active_model.update(&transaction).await?;
    } else {
        models::embedder::ActiveModel {
            name: Set(embedder_name),
            index_id: Set(index_id),
            dimensions: Set(dimensions),
            params: Set(params),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

async fn next_empty_index_name(transaction: &DatabaseTransaction) -> Result<String, DbErr> {
    let names = models::index::Entity::find()
        .all(transaction)
        .await?
        .into_iter()
        .map(|index| index.name)
        .collect::<HashSet<_>>();

    for suffix in 1.. {
        let candidate = if suffix == 1 {
            "新しいインデックス".to_owned()
        } else {
            format!("新しいインデックス ({suffix})")
        };
        if !names.contains(&candidate) {
            return Ok(candidate);
        }
    }

    unreachable!("an unbounded index name sequence must contain an available name")
}

async fn get_index(database: &DatabaseConnection, id: i32) -> Result<Index, DbErr> {
    let model = models::index::Entity::find_by_id(id)
        .one(database)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("index {id} was not created")))?;
    load_index(database, model).await
}

async fn load_index(
    database: &DatabaseConnection,
    model: models::index::Model,
) -> Result<Index, DbErr> {
    let embedder = models::embedder::Entity::find()
        .filter(models::embedder::Column::IndexId.eq(model.id))
        .one(database)
        .await?
        .map(|embedder| Embedder {
            id: embedder.id,
            index_id: embedder.index_id,
            name: embedder.name,
            dimensions: embedder.dimensions,
            params: embedder.params,
        });
    let vector_store = models::vector_store::Entity::find()
        .filter(models::vector_store::Column::IndexId.eq(model.id))
        .one(database)
        .await?
        .map(|vector_store| VectorStore {
            id: vector_store.id,
            index_id: vector_store.index_id,
            name: vector_store.name,
            dimensions: vector_store.dimensions,
            params: vector_store.params,
        });
    let indexers = models::indexer::Entity::find()
        .filter(models::indexer::Column::IndexId.eq(model.id))
        .order_by_asc(models::indexer::Column::Id)
        .all(database)
        .await?;

    let mut dto_indexers = Vec::with_capacity(indexers.len());
    for indexer in indexers {
        dto_indexers.push(load_indexer(database, indexer).await?);
    }

    Ok(Index {
        id: model.id,
        name: model.name,
        description: model.description,
        created_at: model.created_at,
        updated_at: model.updated_at,
        use_mcp_server: model.use_mcp_server,
        embedder,
        vector_store,
        indexers: dto_indexers,
    })
}

async fn load_indexer(
    database: &DatabaseConnection,
    model: models::indexer::Model,
) -> Result<Indexer, DbErr> {
    let file_source = models::file_source::Entity::find()
        .filter(models::file_source::Column::IndexerId.eq(model.id))
        .one(database)
        .await?
        .ok_or_else(|| missing_relation("file source", "indexer", model.id))?;
    let schedule = models::schedule::Entity::find()
        .filter(models::schedule::Column::IndexerId.eq(model.id))
        .one(database)
        .await?
        .ok_or_else(|| missing_relation("schedule", "indexer", model.id))?;
    let files = models::file::Entity::find()
        .filter(models::file::Column::FileSourceId.eq(file_source.id))
        .order_by_asc(models::file::Column::Id)
        .all(database)
        .await?
        .into_iter()
        .map(|file| File {
            id: file.id,
            file_source_id: file.file_source_id,
            name: file.name,
            uri: file.uri,
            content_type: file.content_type,
            size: file.size.to_string(),
            updated_at: file.updated_at,
        })
        .collect();
    let oauth_client = load_oauth_client(database, file_source.id).await?;
    let tasks = models::task::Entity::find()
        .filter(models::task::Column::IndexerId.eq(model.id))
        .order_by_asc(models::task::Column::Id)
        .all(database)
        .await?
        .into_iter()
        .map(|task| Task {
            id: task.id,
            indexer_id: task.indexer_id,
            task_type: task.task_type,
            status: task.status,
            started_at: task.started_at,
            completed_at: task.completed_at,
            state: task.state,
        })
        .collect();

    Ok(Indexer {
        id: model.id,
        index_id: model.index_id,
        extractor_name: model.extractor_name,
        params: model.params,
        last_start: model.last_start,
        last_success: model.last_success,
        last_processed_files: model.last_processed_files,
        file_source: FileSource {
            id: file_source.id,
            indexer_id: file_source.indexer_id,
            name: file_source.name,
            params: file_source.params,
            files,
            oauth_client,
        },
        schedule: Schedule {
            id: schedule.id,
            indexer_id: schedule.indexer_id,
            rfc5545_rrule: schedule.rfc5545_rrule,
            catchup_when_missed: schedule.catchup_when_missed,
        },
        tasks,
    })
}

async fn load_oauth_client(
    database: &DatabaseConnection,
    file_source_id: i32,
) -> Result<Option<OAuthClient>, DbErr> {
    let Some(authentication) = models::oauth_authentication::Entity::find_by_id(file_source_id)
        .one(database)
        .await?
    else {
        return Ok(None);
    };
    let client = models::oauth_client::Entity::find_by_id(authentication.oauth_id)
        .one(database)
        .await?
        .ok_or_else(|| missing_relation("OAuth client", "file source", file_source_id))?;

    Ok(Some(OAuthClient {
        id: client.id,
        name: client.name,
        client_id: client.client_id,
        authorization_endpoint: client.authorization_endpoint,
        token_endpoint: client.token_endpoint,
        refresh_endpoint: client.refresh_endpoint,
        scopes: client.scopes,
    }))
}

fn missing_relation(relation: &str, parent: &str, parent_id: i32) -> DbErr {
    DbErr::Custom(format!("{relation} is missing for {parent} {parent_id}"))
}

#[cfg(test)]
mod tests {
    use sea_orm::EntityTrait;

    use super::{create_empty_index, list_indexes, rename_index};
    use crate::repository::{initialize, models};

    #[tokio::test]
    async fn creates_and_lists_an_empty_index() {
        let database = initialize("sqlite::memory:").await.unwrap();

        let created = create_empty_index(&database).await.unwrap();

        assert_eq!(created.name, "新しいインデックス");
        assert!(created.description.is_empty());
        assert!(created.embedder.is_none());
        assert!(created.vector_store.is_none());
        assert!(created.indexers.is_empty());
        assert!(models::index::Entity::find_by_id(created.id)
            .one(&database)
            .await
            .unwrap()
            .is_some());

        let indexes = list_indexes(&database).await.unwrap();
        assert_eq!(indexes.len(), 1);
        assert_eq!(indexes[0].id, created.id);
    }

    #[tokio::test]
    async fn gives_each_empty_index_a_unique_default_name() {
        let database = initialize("sqlite::memory:").await.unwrap();

        create_empty_index(&database).await.unwrap();
        let second = create_empty_index(&database).await.unwrap();

        assert_eq!(second.name, "新しいインデックス (2)");
    }

    #[tokio::test]
    async fn renames_an_index_in_the_database() {
        let database = initialize("sqlite::memory:").await.unwrap();
        let created = create_empty_index(&database).await.unwrap();

        let renamed = rename_index(&database, created.id, "プロジェクト資料".into())
            .await
            .unwrap();

        assert_eq!(renamed.name, "プロジェクト資料");
        assert_eq!(
            list_indexes(&database).await.unwrap()[0].name,
            "プロジェクト資料"
        );
    }

    #[tokio::test]
    async fn rejects_an_empty_index_name() {
        let database = initialize("sqlite::memory:").await.unwrap();
        let created = create_empty_index(&database).await.unwrap();

        assert!(rename_index(&database, created.id, "  ".into())
            .await
            .is_err());
    }
}
