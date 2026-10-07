use sea_orm::DatabaseConnection;

use crate::{
    dto::models::McpServer,
    server::{
        manager::{end, get_info, start},
        mcp::server::get_server,
    },
};

const MCP_SERVER_NAME: &str = "mcp";

pub async fn get_mcp_server_state() -> Result<Option<McpServer>, String> {
    let info = get_info(MCP_SERVER_NAME);
    match info {
        Ok((t, addr)) => Ok(Some(McpServer {
            port: addr.port(),
            started_at: t,
        })),
        Err(_err) => Ok(None),
    }
}

pub async fn set_mcp_server_running(
    running: bool,
    port: Option<u16>,
    db: &DatabaseConnection,
) -> Result<(), String> {
    let info = get_mcp_server_state().await?;
    let is_running = info.is_some();
    if is_running == running {
        return Ok(());
    } else if running {
        start(MCP_SERVER_NAME, get_server(db.clone()), port)?;
        return Ok(());
    } else {
        end(MCP_SERVER_NAME)?;
        return Ok(());
    }
}
