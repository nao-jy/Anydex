use enum_iterator::all;
use sea_orm::{DatabaseConnection, EntityTrait};
use tauri::State;

use crate::{dto::models::PluginInfo, plugin::entry::PluginType, repository};

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn get_plugin_info() -> Result<Vec<PluginInfo>, String> {
    let ret = all::<PluginType>()
        .map(|f| PluginInfo {
            plugin_type: f.into(),
            path: f.get_dir(),
            plugins: f.list_names(),
        })
        .collect();
    Ok(ret)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn reload_plugins() -> Result<(), String> {
    for p in all::<PluginType>() {
        p.reload()?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn get_param_schema(plugin_type: PluginType, plugin_name: &str) -> Option<String> {
    plugin_type
        .get_param_schema(plugin_name)
        .map(|v| v.to_string())
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
    let _model = repository::models::index::Entity::find_by_id(index_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;

    let embedders = PluginType::Embedder.list_names();
    if !embedders.contains(&embedder_name) {
        return Err(format!("Embedder {embedder_name} was not found"));
    }

    let params: serde_json::Value =
        serde_json::from_str(&params_json).map_err(|err| err.to_string())?;

    repository::plugins::upsert_embedder(&database, index_id, embedder_name, params)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
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
    let _model = repository::models::index::Entity::find_by_id(index_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;

    let plugins = PluginType::VectorStore.list_names();
    if !plugins.contains(&vector_store_name) {
        return Err(format!("Vector store {vector_store_name} was not found"));
    }

    let params: serde_json::Value =
        serde_json::from_str(&params_json).map_err(|err| err.to_string())?;

    repository::plugins::upsert_vector_store(&database, index_id, vector_store_name, params)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
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
    let _model = repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    let names = PluginType::FileExtractor.list_names();
    if !names.contains(&file_extractor_name) {
        return Err(format!(
            "File Extractor {file_extractor_name} was not found"
        ));
    }

    let params: serde_json::Value =
        serde_json::from_str(&params_json).map_err(|err| err.to_string())?;

    repository::plugins::upsert_file_extractor(&database, indexer_id, file_extractor_name, params)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
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
    let _model = repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database.inner())
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    let plugins = PluginType::FileSource.list_names();
    if !plugins.contains(&file_source_name) {
        return Err(format!("File Source {file_source_name} was not found"));
    }

    let params: serde_json::Value =
        serde_json::from_str(&params_json).map_err(|err| err.to_string())?;

    repository::plugins::upsert_file_source(
        &database,
        indexer_id,
        file_source_name,
        oauth_id,
        params,
    )
    .await
    .map_err(|err| err.to_string())?;

    Ok(())
}
