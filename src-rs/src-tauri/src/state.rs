use chrono::{TimeZone, Utc};
use std::sync::Mutex;

/// Mutable process-local state used by the MCP server controls.
#[derive(Debug)]
pub struct McpServerState {
    mcp_port: Mutex<u16>,
    mcp_server_started_at: Mutex<Option<chrono::DateTime<Utc>>>,
}

impl Default for McpServerState {
    fn default() -> Self {
        Self {
            mcp_port: Mutex::new(3001),
            mcp_server_started_at: Mutex::new(Some(utc_datetime(2026, 9, 15, 9, 30))),
        }
    }
}

impl McpServerState {
    pub fn port(&self) -> u16 {
        *self
            .mcp_port
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_port(&self, port: u16) {
        *self
            .mcp_port
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = port;
    }

    pub fn started_at(&self) -> Option<chrono::DateTime<Utc>> {
        *self
            .mcp_server_started_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_running(&self, running: bool) {
        *self
            .mcp_server_started_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            if running { Some(Utc::now()) } else { None };
    }
}

fn utc_datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .expect("mock date must be valid")
}
