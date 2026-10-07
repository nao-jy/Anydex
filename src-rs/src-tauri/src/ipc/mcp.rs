use sea_orm::DatabaseConnection;
use tauri::State;

use crate::usecase;

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn get_mcp_server_state() -> Result<Option<crate::dto::models::McpServer>, String> {
    usecase::mcp::get_mcp_server_state().await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn set_mcp_server_status(
    running: bool,
    port: Option<u16>,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::mcp::set_mcp_server_running(running, port, database.inner()).await
}
