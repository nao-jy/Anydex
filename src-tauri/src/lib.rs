// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chrono::{TimeZone, Utc};
use specta_typescript::Typescript;
use std::{collections::HashMap, path::PathBuf, sync::Mutex, time::Duration};
use tauri::{Manager, State};
use tauri_specta::{collect_commands, collect_events, Builder};

pub mod dto;
pub mod logging;
pub mod repository;
use dto::models::{
    ApplicationSettings, Embedder, FileSource, Index, Indexer, McpServer, PluginDirectory,
    PluginReloadResult, Schedule, SearchResult, Task, VectorStore,
};

/// Mutable process-local state used by the management-screen mocks until the
/// corresponding settings are persisted in the application database.
#[derive(Debug)]
struct MockState {
    mcp_index_overrides: Mutex<HashMap<i32, bool>>,
    mcp_port: Mutex<u16>,
    mcp_server_started_at: Mutex<Option<chrono::DateTime<Utc>>>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            mcp_index_overrides: Mutex::new(HashMap::new()),
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

fn mock_indexer(
    id: i32,
    index_id: i32,
    name: &str,
    uri: &str,
    rule: &str,
    last_start: Option<chrono::DateTime<Utc>>,
    last_success: Option<chrono::DateTime<Utc>>,
    last_processed_files: Option<i32>,
) -> Indexer {
    Indexer {
        id,
        index_id,
        extractor_name: "opendal".into(),
        params: serde_json::json!({}),
        file_source: FileSource {
            id,
            indexer_id: id,
            name: name.into(),
            params: serde_json::json!({ "root_uri": uri }),
            files: vec![],
            oauth_client: None,
        },
        schedule: Schedule {
            id,
            indexer_id: id,
            timezone: "Asia/Tokyo".into(),
            dtstart: utc_datetime(2026, 9, 14, 0, 0),
            rfc5545_rule: rule.into(),
            misfire_policy: "catchup".into(),
        },
        last_start,
        last_success,
        last_processed_files,
        tasks: match id {
            1 => vec![Task {
                id: 101,
                indexer_id: id,
                task_type: "同期".into(),
                status: "running".into(),
                started_at: Some(utc_datetime(2026, 9, 15, 9, 42)),
                completed_at: None,
                state: serde_json::json!({
                    "phase": "embedding",
                    "progress": 68,
                    "processedFiles": 170,
                    "totalFiles": 248,
                }),
            }],
            3 => vec![Task {
                id: 102,
                indexer_id: id,
                task_type: "同期".into(),
                status: "running".into(),
                started_at: Some(utc_datetime(2026, 9, 15, 10, 5)),
                completed_at: None,
                state: serde_json::json!({
                    "phase": "scanning",
                    "progress": 24,
                    "processedFiles": 31,
                    "totalFiles": 127,
                }),
            }],
            _ => vec![],
        },
    }
}

fn mock_indexes() -> Vec<Index> {
    vec![
        Index {
            id: 1,
            name: "社内ナレッジ".into(),
            description: "チーム文書、議事録、設計資料を横断検索".into(),
            created_at: utc_datetime(2026, 9, 1, 0, 0),
            updated_at: utc_datetime(2026, 9, 14, 9, 12),
            use_mcp_server: false,
            embedder: Embedder {
                id: 1,
                index_id: 1,
                name: "Text Embedding v3".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            vector_store: VectorStore {
                id: 1,
                index_id: 1,
                name: "sqlite-vec".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            indexers: vec![
                mock_indexer(
                    1,
                    1,
                    "チーム文書",
                    "s3://example-documents/knowledge-base",
                    "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR",
                    Some(utc_datetime(2026, 9, 14, 0, 12)),
                    Some(utc_datetime(2026, 9, 14, 0, 12)),
                    Some(248),
                ),
                mock_indexer(
                    2,
                    1,
                    "ローカルノート",
                    "fs:///Users/example/Documents/notes",
                    "FREQ=DAILY;INTERVAL=1",
                    Some(utc_datetime(2026, 9, 14, 0, 10)),
                    Some(utc_datetime(2026, 9, 14, 0, 10)),
                    Some(84),
                ),
            ],
        },
        Index {
            id: 2,
            name: "プロダクト資料".into(),
            description: "仕様書、リリースノート、UI 資産".into(),
            created_at: utc_datetime(2026, 8, 22, 0, 0),
            updated_at: utc_datetime(2026, 9, 13, 18, 30),
            use_mcp_server: false,
            embedder: Embedder {
                id: 2,
                index_id: 2,
                name: "Text Embedding v3".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            vector_store: VectorStore {
                id: 2,
                index_id: 2,
                name: "sqlite-vec".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            indexers: vec![mock_indexer(
                3,
                2,
                "プロダクトリポジトリ",
                "fs:///Users/example/work/product/docs",
                "FREQ=DAILY;INTERVAL=1",
                Some(utc_datetime(2026, 9, 14, 0, 15)),
                Some(utc_datetime(2026, 9, 13, 9, 30)),
                Some(127),
            )],
        },
        Index {
            id: 3,
            name: "デザインアセット".into(),
            description: "ブランド画像とデザインガイドライン".into(),
            created_at: utc_datetime(2026, 9, 5, 0, 0),
            updated_at: utc_datetime(2026, 9, 12, 16, 4),
            use_mcp_server: true,
            embedder: Embedder {
                id: 3,
                index_id: 3,
                name: "Text Embedding v3".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            vector_store: VectorStore {
                id: 3,
                index_id: 3,
                name: "sqlite-vec".into(),
                dimensions: 768,
                params: serde_json::json!({}),
            },
            indexers: vec![mock_indexer(
                4,
                3,
                "画像アーカイブ",
                "s3://example-assets/design",
                "FREQ=WEEKLY;BYDAY=FR",
                Some(utc_datetime(2026, 9, 12, 7, 4)),
                None,
                None,
            )],
        },
    ]
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
fn list_indexes(state: State<'_, MockState>) -> Vec<Index> {
    tracing::info!("listing indexes");
    let overrides = state
        .mcp_index_overrides
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let indexes = mock_indexes()
        .into_iter()
        .map(|mut index| {
            if let Some(enabled) = overrides.get(&index.id) {
                index.use_mcp_server = *enabled;
            }
            index
        })
        .collect::<Vec<_>>();
    tracing::info!(count = indexes.len(), "indexes listed");
    indexes
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
fn set_index_mcp_server(index_id: i32, enabled: bool, state: State<'_, MockState>) {
    state
        .mcp_index_overrides
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(index_id, enabled);
    tracing::info!(index_id, enabled, "mock MCP index setting updated");
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(state))]
fn list_mcp_servers(state: State<'_, MockState>) -> Vec<McpServer> {
    let overrides = state
        .mcp_index_overrides
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let enabled_index_count = mock_indexes()
        .iter()
        .filter(|index| {
            overrides
                .get(&index.id)
                .copied()
                .unwrap_or(index.use_mcp_server)
        })
        .count() as i32;
    let port = *state
        .mcp_port
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let started_at = *state
        .mcp_server_started_at
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let is_running = started_at.is_some();

    vec![McpServer {
        name: "Anydex MCP Server".into(),
        host: "127.0.0.1".into(),
        port,
        status: if is_running { "running" } else { "stopped" }.into(),
        started_at,
        enabled_index_count,
    }]
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
async fn list_active_tasks() -> Vec<Task> {
    tokio::time::sleep(Duration::from_millis(300)).await;
    mock_indexes()
        .into_iter()
        .flat_map(|index| index.indexers)
        .flat_map(|indexer| indexer.tasks)
        .filter(|task| task.status == "running")
        .collect()
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
