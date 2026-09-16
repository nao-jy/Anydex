// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, Set,
};
use specta_typescript::Typescript;
use std::{path::PathBuf, sync::Mutex, time::Duration};
use tauri::{Manager, State};
use tauri_specta::{collect_commands, collect_events, Builder};

pub mod dto;
pub mod logging;
pub mod plugin;
pub mod repository;
use dto::models::{
    ApplicationSettings, McpServer, PluginDirectory, PluginReloadResult, SearchResult, Task,
};

/// Mutable process-local state used by the MCP server controls.
#[derive(Debug)]
struct MockState {
    mcp_port: Mutex<u16>,
    mcp_server_started_at: Mutex<Option<chrono::DateTime<Utc>>>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            mcp_port: Mutex::new(3001),
            mcp_server_started_at: Mutex::new(Some(utc_datetime(2026, 9, 15, 9, 30))),
        }
    }
}

fn utc_datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .expect("mock date must be valid")
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
async fn list_indexes(
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<dto::models::Index>, String> {
    tracing::info!("listing indexes");
    let indexes = repository::indexes::list_indexes(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(count = indexes.len(), "indexes listed");
    Ok(indexes)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
async fn create_empty_index(
    database: State<'_, DatabaseConnection>,
) -> Result<dto::models::Index, String> {
    let index = repository::indexes::create_empty_index(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "empty index created");
    Ok(index)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database, name))]
async fn rename_index(
    index_id: i32,
    name: String,
    database: State<'_, DatabaseConnection>,
) -> Result<dto::models::Index, String> {
    let index = repository::indexes::rename_index(database.inner(), index_id, name)
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id = index.id, "index renamed");
    Ok(index)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
async fn set_index_mcp_server(
    index_id: i32,
    enabled: bool,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let model = repository::models::index::Entity::find_by_id(index_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;
    let mut active_model = model.into_active_model();
    active_model.use_mcp_server = Set(enabled);
    active_model.updated_at = Set(Utc::now());
    active_model
        .update(database.inner())
        .await
        .map_err(|error| error.to_string())?;
    tracing::info!(index_id, enabled, "MCP index setting updated");
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state, database))]
async fn list_mcp_servers(
    state: State<'_, MockState>,
    database: State<'_, DatabaseConnection>,
) -> Result<Vec<McpServer>, String> {
    let enabled_index_count = repository::models::index::Entity::find()
        .filter(repository::models::index::Column::UseMcpServer.eq(true))
        .count(database.inner())
        .await
        .map_err(|error| error.to_string())? as i32;
    let port = *state
        .mcp_port
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let started_at = *state
        .mcp_server_started_at
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
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
fn set_mcp_server_port(port: u16, state: State<'_, MockState>) {
    *state
        .mcp_port
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = port;
    tracing::info!(port, "mock MCP server port updated");
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
fn set_mcp_server_running(running: bool, state: State<'_, MockState>) {
    *state
        .mcp_server_started_at
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) =
        if running { Some(Utc::now()) } else { None };
    tracing::info!(running, "mock MCP server state updated");
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument]
async fn list_active_tasks(database: State<'_, DatabaseConnection>) -> Result<Vec<Task>, String> {
    let tasks = repository::models::task::Entity::find()
        .filter(repository::models::task::Column::Status.eq("running"))
        .all(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|task| Task {
            id: task.id,
            indexer_id: task.indexer_id,
            task_type: task.task_type,
            status: task.status,
            started_at: task.started_at,
            completed_at: task.completed_at,
            state: task.state,
        })
        .collect();
    Ok(tasks)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(app))]
fn get_application_settings(app: tauri::AppHandle) -> ApplicationSettings {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let log_directory = app
        .path()
        .app_log_dir()
        .unwrap_or_else(|_| app_data_dir.join("logs"));

    ApplicationSettings {
        database_path: app_data_dir.join("anydex.sqlite3").display().to_string(),
        log_directory: log_directory.display().to_string(),
        plugin_directories: vec![
            PluginDirectory {
                name: "ファイルソース".into(),
                path: app_data_dir
                    .join("plugins/file-sources")
                    .display()
                    .to_string(),
                plugin_count: 1,
            },
            PluginDirectory {
                name: "抽出器".into(),
                path: app_data_dir
                    .join("plugins/extractors")
                    .display()
                    .to_string(),
                plugin_count: 2,
            },
            PluginDirectory {
                name: "ベクトルストア".into(),
                path: app_data_dir
                    .join("plugins/vector-stores")
                    .display()
                    .to_string(),
                plugin_count: 1,
            },
            PluginDirectory {
                name: "ベクトル化".into(),
                path: app_data_dir.join("plugins/embedders").display().to_string(),
                plugin_count: 1,
            },
        ],
    }
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument]
fn reload_plugins() -> PluginReloadResult {
    PluginReloadResult {
        reloaded_at: Utc::now(),
        discovered_plugins: 4,
    }
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(query), fields(query_length))]
async fn search_mock(query: String) -> Vec<SearchResult> {
    tracing::Span::current().record("query_length", query.len());
    tracing::info!("running mock search");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let results = vec![
        SearchResult { title: "2026年度 プロダクトロードマップ".into(), source: "社内ナレッジ / チーム文書".into(), content_type: "PDF".into(), score: "96%".into(), excerpt: "第 3 四半期では検索基盤の刷新を行い、マルチモーダルコンテンツへの対応を段階的に開始します。…".into(), updated_at: "今日 09:12".into() },
        SearchResult { title: "検索体験の設計方針".into(), source: "プロダクト資料 / プロダクトリポジトリ".into(), content_type: "Markdown".into(), score: "91%".into(), excerpt: "検索結果には出所と関連度を明示し、利用者が原文へすぐ遷移できる状態を保つ。ベクトル検索とメタデータフィルタを…".into(), updated_at: "昨日 18:30".into() },
        SearchResult { title: "検索画面ワイヤーフレーム".into(), source: "デザインアセット / 画像アーカイブ".into(), content_type: "PNG".into(), score: "87%".into(), excerpt: "検索ボックス、インデックス選択、結果一覧を含む初期画面のデザイン案。…".into(), updated_at: "09/12 16:04".into() },
    ];
    tracing::info!(count = results.len(), "mock search complete");
    results
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument]
fn greet(name: &str) -> String {
    tracing::debug!(name, "handling greeting");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder: Builder<tauri::Wry> = Builder::new()
        .commands(collect_commands![
            greet,
            list_indexes,
            create_empty_index,
            rename_index,
            set_index_mcp_server,
            list_mcp_servers,
            set_mcp_server_port,
            set_mcp_server_running,
            list_active_tasks,
            get_application_settings,
            reload_plugins,
            search_mock
        ])
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .setup(|app| {
            app.manage(MockState::default());
            let (log_guard, log_dir) = logging::init(app)?;
            app.manage(log_guard);
            tracing::info!(path = %log_dir.display(), "file logging initialized");

            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("anydex.sqlite3");
            let database_url = repository::sqlite_database_url(&database_path)?;
            let database = tauri::async_runtime::block_on(repository::initialize(&database_url))?;
            app.manage(database);
            tracing::info!(path = %database_path.display(), "application database initialized");
            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_indexes,
            create_empty_index,
            rename_index,
            set_index_mcp_server,
            list_mcp_servers,
            set_mcp_server_port,
            set_mcp_server_running,
            list_active_tasks,
            get_application_settings,
            reload_plugins,
            search_mock
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_typescript_bindings() {
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
        let builder: Builder<tauri::Wry> = Builder::new()
            .commands(collect_commands![
                greet,
                list_indexes,
                create_empty_index,
                rename_index,
                set_index_mcp_server,
                list_mcp_servers,
                set_mcp_server_port,
                set_mcp_server_running,
                list_active_tasks,
                get_application_settings,
                reload_plugins,
                search_mock
            ])
            .events(collect_events![]);

        builder
            .export(Typescript::default(), &output)
            .expect("DTOs must export to TypeScript");
    }
}
