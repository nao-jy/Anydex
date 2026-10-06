use std::time::Duration;

use crate::dto::models::SearchResult;

pub async fn search_mock(query: String) -> Vec<SearchResult> {
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

pub fn greet(name: &str) -> String {
    tracing::debug!(name, "handling greeting");
    format!("Hello, {}! You've been greeted from Rust!", name)
}
