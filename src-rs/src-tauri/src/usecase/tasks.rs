use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{dto::models::Task, repository};

pub async fn list_active_tasks(database: &DatabaseConnection) -> Result<Vec<Task>, String> {
    let tasks = repository::sqlite::models::task::Entity::find()
        .filter(repository::sqlite::models::task::Column::Status.eq("running"))
        .all(database)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|task| Task {
            id: task.id,
            index_id: task.index_id,
            task_type: task.task_type,
            status: task.status,
            started_at: task.started_at,
            completed_at: task.completed_at,
            state: task.state,
        })
        .collect();
    Ok(tasks)
}
