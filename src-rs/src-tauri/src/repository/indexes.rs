//! Persistence operations for the index aggregate.

use std::collections::HashSet;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::option::Option::{None, Some};

use crate::{
    dto::models::{
        Embedder, File, FileExtractor, FileSource, Index, Indexer, Schedule, Task, VectorStore,
    },
    repository::{models, oauth::load_oauth_client},
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

pub async fn put_description(
    database: &DatabaseConnection,
    index_id: i32,
    description: String,
) -> Result<Index, DbErr> {
    let description = description.trim();

    let model = models::index::Entity::find_by_id(index_id)
        .one(database)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("index {index_id} was not found")))?;
    if model.description == description {
        return load_index(database, model).await;
    }

    let mut active_model = model.into_active_model();
    active_model.description = Set(description.to_owned());
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
        .await?;
    let file_extractor = models::file_extractor::Entity::find()
        .filter(models::file_extractor::Column::IndexerId.eq(model.id))
        .one(database)
        .await?;
    let schedule = models::schedule::Entity::find()
        .filter(models::schedule::Column::IndexerId.eq(model.id))
        .one(database)
        .await?;
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

    let file_source_dto = match file_source {
        Some(file_source_some) => {
            let files = models::file::Entity::find()
                .filter(models::file::Column::FileSourceId.eq(file_source_some.id))
                .order_by_asc(models::file::Column::Id)
                .all(database)
                .await?
                .into_iter()
                .map(|file| File {
                    id: file.id,
                    file_source_id: file.file_source_id,
                    name: file.name,
                    file_uri: file.file_uri,
                    file_path: file.file_path,
                    file_dir: file.file_dir,
                    file_name: file.file_name,
                    content_type: file.content_type,
                    size: file.size.to_string(),
                    updated_at: file.updated_at,
                })
                .collect();
            let oauth_client = if let Some(oauth_id) = file_source_some.oauth_id {
                load_oauth_client(database, oauth_id).await?
            } else {
                None
            };
            Some(FileSource {
                id: file_source_some.id,
                indexer_id: file_source_some.indexer_id,
                name: file_source_some.name,
                params: file_source_some.params,
                files,
                oauth_client,
            })
        }
        None => None,
    };

    let file_ex_dto = match file_extractor {
        Some(file_ex_some) => Some(FileExtractor {
            id: file_ex_some.id,
            indexer_id: file_ex_some.indexer_id,
            name: file_ex_some.name,
            params: file_ex_some.params,
        }),
        None => None,
    };

    let schedule_dto = match schedule {
        Some(schedule_some) => Some(Schedule {
            id: schedule_some.id,
            indexer_id: schedule_some.indexer_id,
            rfc5545_rrule: schedule_some.rfc5545_rrule,
            catchup_when_missed: schedule_some.catchup_when_missed,
        }),
        None => None,
    };

    Ok(Indexer {
        id: model.id,
        index_id: model.index_id,
        last_start: model.last_start,
        last_success: model.last_success,
        last_processed_files: model.last_processed_files,
        file_source: file_source_dto,
        file_extractor: file_ex_dto,
        schedule: schedule_dto,
        tasks,
    })
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
