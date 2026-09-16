use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use tauri::State;

use crate::{dto::models::McpServer, repository, state::McpServerState};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state, database))]
pub async fn list_mcp_servers(
    state: State<'_, McpServerState>,
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<McpServer>, String> {
    let enabled_index_count = repository::models::index::Entity::find()
        .filter(repository::models::index::Column::UseMcpServer.eq(true))
        .count(database.inner())
        .await
        .map_err(|error| error.to_string())? as i32;
    let port = state.port();
    let started_at = state.started_at();
    let is_running = started_at.is_some();

    Ok(vec![McpServer {
        name: "Anydex MCP Server".into(),
        host: "127.0.0.1".into(),
        port,
        status: if is_running { "running" } else { "stopped" }.into(),
        started_at,
        enabled_index_count,
    }])
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
pub fn set_mcp_server_port(port: u16, state: State<'_, McpServerState>) {
    state.set_port(port);
    tracing::info!(port, "mock MCP server port updated");
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
pub fn set_mcp_server_running(running: bool, state: State<'_, McpServerState>) {
    state.set_running(running);
    tracing::info!(running, "mock MCP server state updated");
}
