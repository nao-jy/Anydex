use uuid::Uuid;

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
pub fn get_param_schema(plugin_type: PluginType, plugin_name: &str) -> Option<String> {
    usecase::plugins::get_param_schema(plugin_type, plugin_name)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn upsert_embedder(
    index_id: Uuid,
    embedder_name: String,
    params_json: String,
) -> Result<(), String> {
    usecase::plugins::upsert_embedder(index_id, embedder_name, params_json).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn upsert_vector_store(
    index_id: Uuid,
    vector_store_name: String,
    params_json: String,
) -> Result<(), String> {
    usecase::plugins::upsert_vector_store(index_id, vector_store_name, params_json).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn upsert_file_extractor(
    index_id: Uuid,
    file_extractor_name: String,
    params_json: String,
) -> Result<(), String> {
    usecase::plugins::upsert_file_extractor(index_id, file_extractor_name, params_json).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn upsert_file_source(
    index_id: Uuid,
    file_source_name: String,
    params_json: String,
    oauth_id: Option<Uuid>,
) -> Result<(), String> {
    usecase::plugins::upsert_file_source(index_id, file_source_name, oauth_id, params_json).await
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub async fn test_file_source(
    file_source_name: String,
    params_json: String,
    oauth_id: Option<Uuid>,
) -> Result<(), String> {
    usecase::plugins::test_file_source(file_source_name, params_json, oauth_id).await
}
