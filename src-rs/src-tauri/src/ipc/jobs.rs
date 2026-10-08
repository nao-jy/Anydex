use sea_orm::DatabaseConnection;
use tauri::State;

use crate::usecase;

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn list_active_tasks(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<crate::dto::models::Job>, String> {
    usecase::jobs::list_active_tasks(database.inner()).await
}
