use crate::{dto::models::McpServer, server::manager::get_info};

pub async fn get_mcp_server_state() -> Result<Option<McpServer>, String> {
    let info = get_info("mcp");
    match info {
        Ok((t, addr)) => Ok(Some(McpServer {
            port: addr.port(),
            started_at: t,
        })),
        Err(_err) => Ok(None),
    }
}

pub async fn set_mcp_server_running(running: bool) -> Result<(), String> {
    Ok(())
}
