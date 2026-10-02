use std::fs;

use enum_iterator::all;
use sea_orm::{sqlx::types::uuid, DatabaseConnection, EntityTrait};
use tauri::State;
use uuid::Uuid;

use crate::{
    dto::models::PluginInfo,
    plugin::{
        entry::{PluginType, FILE_SOURCE_MGR},
        file_source::interface::FileSource,
    },
    repository,
    server::oauth::client::do_oauth_pkce,
};

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

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn test_file_source(
    file_source_name: String,
    params_json: String,
    oauth_id: Option<i32>,
    database: State<'_, DatabaseConnection>,
) -> Result<(), String> {
    let param = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    let auth_result = match oauth_id {
        Some(id) => {
            let oauth = repository::models::oauth_client::Entity::find_by_id(id)
                .one(database.inner())
                .await
                .map_err(|err| err.to_string())?
                .ok_or_else(|| format!("OAuth client {id} was not found"))?;
            let scopes: Vec<String> = oauth
                .scopes
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            let access_token = do_oauth_pkce(
                &oauth.client_id,
                oauth.client_sercret,
                &oauth.authorization_endpoint,
                &oauth.token_endpoint,
                &scopes,
            )
            .await
            .map_err(|err| err.to_string())?;
            Some(access_token)
        }
        None => None,
    };
    let access_token = match auth_result {
        Some(token) => Some(token.access_token.secret().to_string()),
        None => None,
    };
    let download_dir = std::env::temp_dir()
        .join(Uuid::new_v4().to_string())
        .to_string_lossy()
        .to_string();
    // Acquire the synchronous mutex only after all awaits, so its guard is
    // never held across an await point in this Send future.
    let mut mgr_mtx = FILE_SOURCE_MGR.lock().unwrap();
    let mgr = mgr_mtx.as_mut().unwrap();
    let plugin = mgr.get_plugin(&file_source_name).ok_or_else(|| {
        format!("File Source plugin {file_source_name} was not found in the plugin manager")
    })?;
    let handler = plugin
        .get_handler(&param, &[], access_token, &download_dir)
        .map_err(|err| err.to_string())?;
    let _file = plugin
        .next(handler.handler)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| format!("File not found"))?;
    fs::remove_dir_all(download_dir).map_err(|err| err.to_string())?;
    Ok(())
}
