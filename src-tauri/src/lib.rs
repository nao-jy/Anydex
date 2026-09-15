// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use chrono::{TimeZone, Utc};
use specta_typescript::Typescript;
use std::time::Duration;
use tauri_specta::{collect_commands, collect_events, Builder};

pub mod dto;
pub mod repository;
use dto::models::{Embedder, FileSource, Index, Indexer, Schedule, SearchResult, VectorStore};

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
        tasks: vec![],
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
async fn list_indexes() -> Vec<Index> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    mock_indexes()
}

#[tauri::command]
#[specta::specta]
async fn search_mock(_query: String) -> Vec<SearchResult> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    vec![
        SearchResult { title: "2026年度 プロダクトロードマップ".into(), source: "社内ナレッジ / チーム文書".into(), content_type: "PDF".into(), score: "96%".into(), excerpt: "第 3 四半期では検索基盤の刷新を行い、マルチモーダルコンテンツへの対応を段階的に開始します。…".into(), updated_at: "今日 09:12".into() },
        SearchResult { title: "検索体験の設計方針".into(), source: "プロダクト資料 / プロダクトリポジトリ".into(), content_type: "Markdown".into(), score: "91%".into(), excerpt: "検索結果には出所と関連度を明示し、利用者が原文へすぐ遷移できる状態を保つ。ベクトル検索とメタデータフィルタを…".into(), updated_at: "昨日 18:30".into() },
        SearchResult { title: "検索画面ワイヤーフレーム".into(), source: "デザインアセット / 画像アーカイブ".into(), content_type: "PNG".into(), score: "87%".into(), excerpt: "検索ボックス、インデックス選択、結果一覧を含む初期画面のデザイン案。…".into(), updated_at: "09/12 16:04".into() },
    ]
}

#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder: Builder<tauri::Wry> = Builder::new()
        .commands(collect_commands![greet, list_indexes, search_mock])
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_indexes, search_mock])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_typescript_bindings() {
        let output = std::env::temp_dir().join("anydex-bindings-export-test.ts");
        let builder: Builder<tauri::Wry> = Builder::new()
            .commands(collect_commands![greet, list_indexes, search_mock])
            .events(collect_events![]);

        builder
            .export(Typescript::default(), &output)
            .expect("DTOs must export to TypeScript");

        std::fs::remove_file(output).expect("test binding file must be removable");
    }
}
