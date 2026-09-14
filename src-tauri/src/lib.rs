// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, collect_events, Builder};
use serde::Serialize;
use std::time::Duration;

pub mod models;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct MockFileSource {
    name: String,
    source_type: String,
    path: String,
    files: u32,
    last_sync: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct IndexOverview {
    id: String,
    name: String,
    description: String,
    created_at: String,
    updated_at: String,
    status: String,
    vectorizer: String,
    store: String,
    chunks: String,
    sources: Vec<MockFileSource>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    title: String,
    source: String,
    content_type: String,
    score: String,
    excerpt: String,
    updated_at: String,
}

fn mock_indexes() -> Vec<IndexOverview> {
    vec![
        IndexOverview {
            id: "idx_01".into(), name: "社内ナレッジ".into(), description: "チーム文書、議事録、設計資料を横断検索".into(), created_at: "2026/09/01".into(), updated_at: "2026/09/14 09:12".into(), status: "同期済み".into(), vectorizer: "Multimodal Embedding v1".into(), store: "SQLite Vector（ローカル）".into(), chunks: "4,281".into(),
            sources: vec![
                MockFileSource { name: "チーム文書".into(), source_type: "Amazon S3".into(), path: "s3://example-documents/knowledge-base".into(), files: 248, last_sync: "今日 09:12".into() },
                MockFileSource { name: "ローカルノート".into(), source_type: "Filesystem".into(), path: "~/Documents/notes".into(), files: 84, last_sync: "今日 09:10".into() },
            ],
        },
        IndexOverview {
            id: "idx_02".into(), name: "プロダクト資料".into(), description: "仕様書、リリースノート、UI 資産".into(), created_at: "2026/08/22".into(), updated_at: "2026/09/13 18:30".into(), status: "同期中".into(), vectorizer: "Text Embedding v3".into(), store: "SQLite Vector（ローカル）".into(), chunks: "1,926".into(),
            sources: vec![MockFileSource { name: "プロダクトリポジトリ".into(), source_type: "Filesystem".into(), path: "~/work/product/docs".into(), files: 127, last_sync: "同期中".into() }],
        },
        IndexOverview {
            id: "idx_03".into(), name: "デザインアセット".into(), description: "ブランド画像とデザインガイドライン".into(), created_at: "2026/09/05".into(), updated_at: "2026/09/12 16:04".into(), status: "エラー".into(), vectorizer: "Vision Embedding v2".into(), store: "SQLite Vector（ローカル）".into(), chunks: "3,740".into(),
            sources: vec![MockFileSource { name: "画像アーカイブ".into(), source_type: "Amazon S3".into(), path: "s3://example-assets/design".into(), files: 312, last_sync: "09/12 16:04".into() }],
        },
    ]
}

#[tauri::command]
#[specta::specta]
fn list_indexes() -> Vec<IndexOverview> { mock_indexes() }

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
