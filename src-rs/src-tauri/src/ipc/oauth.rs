use sea_orm::DatabaseConnection;
use tauri::State;
use uuid::Uuid;

use crate::{repository::json::models::OAuthClient, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn list_oauth_client() -> Result<Vec<OAuthClient>, String> {
    usecase::oauth::list_oauth_clients().await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn create_empty_oauth_client() -> Result<(), String> {
    usecase::oauth::create_empty_oauth_client().await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn update_oauth_client(data: OAuthClient) -> Result<(), String> {
    usecase::oauth::update_oauth_client(data).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn delete_oauth_client(
    oauth_id: Uuid,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::oauth::delete_oauth_client(oauth_id, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn test_oauth(client: OAuthClient) -> Result<(), String> {
    usecase::oauth::test_oauth(client).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn abort_oauth(client_id: &str) -> Result<(), String> {
    usecase::oauth::abort_oauth(client_id)
}
