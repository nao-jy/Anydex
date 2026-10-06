use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};

use crate::{dto::models::McpServer, repository, state::McpServerState};

pub async fn list_mcp_servers(
    state: &McpServerState,
    database: &DatabaseConnection,
) -> Result<Vec<McpServer>, String> {
    let enabled_index_count = repository::models::index::Entity::find()
        .filter(repository::models::index::Column::UseMcpServer.eq(true))
        .count(database)
        .await
        .map_err(|error| error.to_string())? as i32;
    let started_at = state.started_at();
    Ok(vec![McpServer {
        name: "Anydex MCP Server".into(),
        host: "127.0.0.1".into(),
        port: state.port(),
        status: if started_at.is_some() {
            "running"
        } else {
            "stopped"
        }
        .into(),
        started_at,
        enabled_index_count,
    }])
}

pub fn set_mcp_server_port(port: u16, state: &McpServerState) {
    state.set_port(port);
    tracing::info!(port, "mock MCP server port updated");
}

pub fn set_mcp_server_running(running: bool, state: &McpServerState) {
    state.set_running(running);
    tracing::info!(running, "mock MCP server state updated");
}
