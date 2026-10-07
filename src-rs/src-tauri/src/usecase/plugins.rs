use std::fs;

use enum_iterator::all;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::{
    dto::models::PluginInfo,
    plugin::{
        entry::{PluginType, FILE_SOURCE_MGR},
        file_source::interface::FileSource,
    },
    repository,
    server::oauth::client::{do_oauth_pkce, revoke_access_token, revoke_refresh_token},
};

pub fn get_plugin_info() -> Result<Vec<PluginInfo>, String> {
    Ok(all::<PluginType>()
        .map(|plugin_type| PluginInfo {
            plugin_type: plugin_type.into(),
            path: plugin_type.get_dir(),
            plugins: plugin_type.list_names(),
        })
        .collect())
}

pub fn reload_plugins() -> Result<(), String> {
    for plugin_type in all::<PluginType>() {
        plugin_type.reload()?;
    }
    Ok(())
}

pub fn get_param_schema(plugin_type: PluginType, plugin_name: &str) -> Option<String> {
    plugin_type
        .get_param_schema(plugin_name)
        .map(|value| value.to_string())
}

pub async fn upsert_embedder(
    index_id: i32,
    embedder_name: String,
    params_json: String,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::index::Entity::find_by_id(index_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;

    if !PluginType::Embedder.list_names().contains(&embedder_name) {
        return Err(format!("Embedder {embedder_name} was not found"));
    }
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    repository::plugins::upsert_embedder(database, index_id, embedder_name, params)
        .await
        .map_err(|err| err.to_string())
}

pub async fn upsert_vector_store(
    index_id: i32,
    vector_store_name: String,
    params_json: String,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::index::Entity::find_by_id(index_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("index {index_id} was not found"))?;

    if !PluginType::VectorStore
        .list_names()
        .contains(&vector_store_name)
    {
        return Err(format!("Vector store {vector_store_name} was not found"));
    }
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    repository::plugins::upsert_vector_store(database, index_id, vector_store_name, params)
        .await
        .map_err(|err| err.to_string())
}

pub async fn upsert_file_extractor(
    indexer_id: i32,
    file_extractor_name: String,
    params_json: String,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    if !PluginType::FileExtractor
        .list_names()
        .contains(&file_extractor_name)
    {
        return Err(format!(
            "File Extractor {file_extractor_name} was not found"
        ));
    }
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    repository::plugins::upsert_file_extractor(database, indexer_id, file_extractor_name, params)
        .await
        .map_err(|err| err.to_string())
}

pub async fn upsert_file_source(
    indexer_id: i32,
    file_source_name: String,
    params_json: String,
    oauth_id: Option<i32>,
    database: &DatabaseConnection,
) -> Result<(), String> {
    repository::models::indexer::Entity::find_by_id(indexer_id)
        .one(database)
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("indexer {indexer_id} was not found"))?;

    if !PluginType::FileSource
        .list_names()
        .contains(&file_source_name)
    {
        return Err(format!("File Source {file_source_name} was not found"));
    }
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    repository::plugins::upsert_file_source(
        database,
        indexer_id,
        file_source_name,
        oauth_id,
        params,
    )
    .await
    .map_err(|err| err.to_string())
}

pub async fn test_file_source(
    file_source_name: String,
    params_json: String,
    oauth_id: Option<i32>,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    let oauth = match oauth_id {
        Some(id) => Some(
            repository::models::oauth_client::Entity::find_by_id(id)
                .one(database)
                .await
                .map_err(|err| err.to_string())?
                .ok_or_else(|| format!("OAuth client {id} was not found"))?,
        ),
        None => None,
    };
    let auth_result = match oauth.as_ref() {
        Some(model) => {
            let scopes: Vec<String> = model
                .scopes
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_owned())
                .collect();
            Some(
                do_oauth_pkce(
                    &model.client_id,
                    model.client_sercret.clone(),
                    &model.authorization_endpoint,
                    &model.token_endpoint,
                    &scopes,
                )
                .await
                .map_err(|err| err.to_string())?,
            )
        }
        None => None,
    };
    let access_token = auth_result
        .as_ref()
        .map(|token| token.access_token.secret().to_string());
    let download_dir = std::env::temp_dir()
        .join(Uuid::new_v4().to_string())
        .to_string_lossy()
        .to_string();
    let plugin_result = tokio::task::spawn_blocking(move || {
        let mut manager = FILE_SOURCE_MGR.lock().unwrap();
        let manager = manager
            .as_mut()
            .ok_or_else(|| "file source plugin manager is not initialized".to_owned())?;
        let plugin = manager.get_plugin(&file_source_name).ok_or_else(|| {
            format!("File Source plugin {file_source_name} was not found in the plugin manager")
        })?;
        let handler = plugin.get_handler(&params).map_err(|err| err.to_string())?;
        let _file = plugin
            .get_diff(handler.handler, &[], access_token)
            .map_err(|err| err.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|err| format!("file source test task failed: {err}"))
    .and_then(|result| result);

    let cleanup_result = fs::remove_dir_all(&download_dir).map_err(|err| err.to_string());
    let result = match (plugin_result, cleanup_result) {
        (Err(err), Err(cleanup_err)) => {
            tracing::warn!("failed to remove file source test directory: {cleanup_err}");
            Err(err)
        }
        (Err(err), _) => Err(err),
        (Ok(()), Err(err)) => Err(err),
        (Ok(()), Ok(())) => Ok(()),
    };

    if let (Some(auth_result), Some(model)) = (auth_result, oauth.as_ref()) {
        if let Some(revocation_endpoint) = model.revocation_endpoint.as_ref() {
            let _ = revoke_access_token(
                &model.client_id,
                model.client_sercret.clone(),
                auth_result.access_token.secret(),
                revocation_endpoint,
            )
            .await;
            if let Some(refresh_token) = auth_result.refresh_token.as_ref() {
                let _ = revoke_refresh_token(
                    &model.client_id,
                    model.client_sercret.clone(),
                    refresh_token.secret(),
                    revocation_endpoint,
                )
                .await;
            }
        }
    }
    result
}
