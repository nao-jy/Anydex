use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tauri::State;

use crate::dto::models::Task;

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_active_tasks(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<Task>, String> {
    let tasks = crate::repository::models::task::Entity::find()
        .filter(crate::repository::models::task::Column::Status.eq("running"))
        .all(database.inner())
        .await
        .map_err(|error| error.to_string())?
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
    Ok(tasks)
}
