use chrono::Utc;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::Value;

use crate::{
    bg_job::entry::JobType,
    dto,
    repository::{
        self,
        sqlite::models::job::{STATUS_PENDING, STATUS_RUNNING},
    },
};

pub async fn list_active_jobs(
    database: &DatabaseConnection,
) -> Result<Vec<dto::models::Job>, String> {
    let tasks = repository::sqlite::models::job::Entity::find()
        .filter(
            repository::sqlite::models::job::Column::Status
                .eq(STATUS_PENDING)
                .or(repository::sqlite::models::job::Column::Status.eq(STATUS_RUNNING)),
        )
        .all(database)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|task| dto::models::Job {
            id: task.id.to_string(),
            index_id: task.index_id,
            job_type: task.job_type,
            status: task.status,
            progress_info: task.progress_info,
            priority: task.priority,
            attempts: task.attempts,
            max_attempts: task.max_attempts,
            run_at: task.run_at,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
        .collect();
    Ok(tasks)
}

pub struct JobInfo {
    pub job_type: JobType,
    pub args: Value,
}

pub async fn add_jobs(
    database: &DatabaseConnection,
    index_id: uuid::Uuid,
    jobs: Vec<JobInfo>,
) -> Result<(), String> {
    let now = Utc::now();
    let models: Vec<repository::sqlite::models::job::ActiveModel> = jobs
        .iter()
        .map(|v| repository::sqlite::models::job::ActiveModel {
            index_id: Set(index_id.to_string()),
            job_type: Set(v.job_type.to_string()),
            payload: Set(v.args.clone()),
            status: Set(STATUS_PENDING.to_string()),
            progress_info: Set("待機中".to_owned()),
            priority: Set(0),
            attempts: Set(0),
            max_attempts: Set(3),
            run_at: Set(now),
            last_error: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..repository::sqlite::models::job::ActiveModel::default()
        })
        .collect();
    repository::sqlite::models::job::Entity::insert_many(models)
        .exec_without_returning(database)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub async fn get_last_indexed_date(
    database: &DatabaseConnection,
    index_id: uuid::Uuid,
) -> Result<Option<dto::models::IndexingHistory>, String> {
    let result = repository::sqlite::models::indexing_history::Entity::find()
        .filter(
            repository::sqlite::models::indexing_history::Column::IndexId.eq(index_id.to_string()),
        )
        .order_by_desc(repository::sqlite::models::indexing_history::Column::StartedAt)
        .one(database)
        .await
        .map_err(|err| err.to_string())?
        .map(|v| dto::models::IndexingHistory {
            id: v.id,
            index_id: v.index_id,
            started_at: v.started_at,
        });
    Ok(result)
}
