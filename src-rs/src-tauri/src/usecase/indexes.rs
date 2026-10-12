use chrono::Utc;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    bg_job::{entry::JobType, job::collect_diff::CollectDiffArgs},
    dto::models::IndexWithLastHistory,
    repository::{
        self,
        json::{
            load,
            models::{Index, Schedule},
            save,
        },
        sqlite::{
            self,
            models::job::{STATUS_PENDING, STATUS_RUNNING},
        },
    },
    usecase::{
        jobs::{add_jobs, JobInfo},
        oauth::delete_authentication,
        plugins::test_file_source,
    },
};

pub async fn list_indexes(db: &DatabaseConnection) -> Result<Vec<IndexWithLastHistory>, String> {
    let userdata = load()?;
    let histories = sqlite::models::indexing_history::Entity::find()
        .order_by_desc(sqlite::models::indexing_history::Column::StartedAt)
        .order_by_desc(sqlite::models::indexing_history::Column::Id)
        .all(db)
        .await
        .map_err(|err| err.to_string())?;
    let mut last_started_at = HashMap::new();
    for history in histories {
        last_started_at
            .entry(history.index_id)
            .or_insert(history.started_at);
    }

    Ok(userdata
        .indexes
        .iter()
        .map(|index| IndexWithLastHistory {
            last_indexing_started_at: last_started_at.get(&index.id.to_string()).copied(),
            index: index.clone(),
        })
        .collect())
}

pub async fn create_empty_index() -> Result<(), String> {
    let mut userdata = load()?;
    userdata.indexes.push(Index::default());
    save(&userdata)
}

pub async fn rename_index(index_id: Uuid, name: String) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.name = name;
            break;
        }
    }
    save(&userdata)
}

pub async fn put_description(index_id: Uuid, description: String) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.description = description;
            break;
        }
    }
    save(&userdata)
}

pub async fn set_index_mcp_server(index_id: Uuid, enabled: bool) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.use_mcp_server = enabled;
            break;
        }
    }
    save(&userdata)
}

pub async fn delete_index(index_id: Uuid, database: &DatabaseConnection) -> Result<(), String> {
    let transaction = database.begin().await.map_err(|err| err.to_string())?;

    delete_authentication(index_id, &transaction).await?;

    let _result = sqlite::models::file::Entity::delete_many()
        .filter(sqlite::models::file::Column::IndexId.eq(index_id.to_string()))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    let _result = sqlite::models::job::Entity::delete_many()
        .filter(sqlite::models::job::Column::IndexId.eq(index_id.to_string()))
        .exec(&transaction)
        .await
        .map_err(|err| err.to_string())?;

    transaction.commit().await.map_err(|err| err.to_string())?;

    let mut userdata = load()?;
    userdata.indexes = userdata
        .indexes
        .iter()
        .filter(|index| index.id != index_id)
        .cloned()
        .collect();
    save(&userdata)
}

pub async fn upsert_schedule(index_id: Uuid, schedule: Option<Schedule>) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.schedule = schedule;
            break;
        }
    }
    save(&userdata)
}

pub async fn start_indexing(index_id: Uuid, database: &DatabaseConnection) -> Result<(), String> {
    if is_indexing_running(index_id, database).await? {
        return Err("already indexing".into());
    }
    is_index_ready(index_id, database).await?;

    let job = JobInfo {
        job_type: JobType::CollectDiff,
        args: serde_json::to_value(CollectDiffArgs { index_id }).unwrap(),
    };
    add_jobs(database, index_id, vec![job]).await?;
    let now = Utc::now();
    let history = repository::sqlite::models::indexing_history::ActiveModel {
        index_id: Set(index_id.to_string()),
        started_at: Set(now),
        ..repository::sqlite::models::indexing_history::ActiveModel::default()
    };
    repository::sqlite::models::indexing_history::Entity::insert(history)
        .exec(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn is_indexing_running(
    index_id: Uuid,
    database: &DatabaseConnection,
) -> Result<bool, String> {
    Ok(sqlite::models::job::Entity::find()
        .filter(sqlite::models::job::Column::IndexId.eq(index_id.to_string()))
        .filter(sqlite::models::job::Column::Status.is_in(vec![STATUS_PENDING, STATUS_RUNNING]))
        .limit(1)
        .one(database)
        .await
        .map_err(|err| err.to_string())?
        .is_some())
}

pub async fn is_index_ready(index_id: Uuid, database: &DatabaseConnection) -> Result<(), String> {
    let userdata = load()?;
    let index = userdata
        .indexes
        .iter()
        .find(|v| v.id == index_id)
        .ok_or("index not found")?;
    if let Some(v) = &index.file_source {
        test_file_source(index_id, v.name.to_owned(), v.params.to_string(), database).await?;
    } else {
        return Err("file source is not set".into());
    }
    if let Some(_v) = &index.file_extractor {
    } else {
        return Err("file extractor is not set".into());
    }
    if let Some(_v) = &index.embedder {
    } else {
        return Err("embedder is not set".into());
    }
    if let Some(_v) = &index.vector_store {
    } else {
        return Err("vector store is not set".into());
    }
    Ok(())
}
