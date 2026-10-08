use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter};

use crate::{
    dto,
    repository::{
        self,
        sqlite::models::job::{STATUS_PENDING, STATUS_RUNNING},
    },
};

pub async fn list_active_tasks(
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
            id: task.id,
            index_id: task.index_id,
            job_type: task.job_type,
            status: task.status,
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
