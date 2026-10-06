use sea_orm::DatabaseConnection;
use tauri::State;

use crate::{plugin::entry::PluginType, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn get_plugin_info() -> Result<Vec<crate::dto::models::PluginInfo>, String> {
    usecase::plugins::get_plugin_info()
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn reload_plugins() -> Result<(), String> {
    usecase::plugins::reload_plugins()
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn get_param_schema(plugin_type: PluginType, plugin_name: &str) -> Option<String> {
    usecase::plugins::get_param_schema(plugin_type, plugin_name)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_embedder(
    index_id: i32,
    embedder_name: String,
    params_json: String,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::plugins::upsert_embedder(index_id, embedder_name, params_json, database.inner()).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_vector_store(
    index_id: i32,
    vector_store_name: String,
    params_json: String,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::plugins::upsert_vector_store(
        index_id,
        vector_store_name,
        params_json,
        database.inner(),
    )
    .await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_file_extractor(
    indexer_id: i32,
    file_extractor_name: String,
    params_json: String,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::plugins::upsert_file_extractor(
        indexer_id,
        file_extractor_name,
        params_json,
        database.inner(),
    )
    .await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_file_source(
    indexer_id: i32,
    file_source_name: String,
    params_json: String,
    oauth_id: Option<i32>,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::plugins::upsert_file_source(
        indexer_id,
        file_source_name,
        params_json,
        oauth_id,
        database.inner(),
    )
    .await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn test_file_source(
    file_source_name: String,
    params_json: String,
    oauth_id: Option<i32>,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    usecase::plugins::test_file_source(file_source_name, params_json, oauth_id, database.inner())
        .await
}
