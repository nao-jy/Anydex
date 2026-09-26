use sea_orm::DatabaseConnection;
use tauri::State;

use crate::dto::models::OAuthClient;
use crate::repository;

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn list_oauth_client(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<OAuthClient>, String> {
    let ret = repository::oauth::list_oauth_client(database.inner())
        .await
        .map_err(|err| err.to_string())?;
    Ok(ret)
}
