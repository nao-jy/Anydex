use sea_orm::DatabaseConnection;
use tauri::State;

use crate::{state::McpServerState, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state, database))]
pub async fn list_mcp_servers(
    state: State<'_, McpServerState>,
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<crate::dto::models::McpServer>, String> {
    usecase::mcp::list_mcp_servers(state.inner(), database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
pub fn set_mcp_server_port(port: u16, state: State<'_, McpServerState>) {
    usecase::mcp::set_mcp_server_port(port, state.inner());
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
pub fn set_mcp_server_running(running: bool, state: State<'_, McpServerState>) {
    usecase::mcp::set_mcp_server_running(running, state.inner());
}
