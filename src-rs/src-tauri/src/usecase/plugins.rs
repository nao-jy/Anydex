use std::fs;

use enum_iterator::all;
use uuid::Uuid;

use crate::{
    dto::models::PluginInfo,
    plugin::{
        entry::{PluginType, FILE_SOURCE_MGR},
        file_source::interface::FileSource,
    },
    repository::json::{
        load,
        models::{self, *},
        save,
    },
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

pub fn get_param_schema(plugin_type: PluginType, plugin_name: &str) -> Option<String> {
    plugin_type
        .get_param_schema(plugin_name)
        .map(|value| value.to_string())
}

pub async fn upsert_embedder(
    index_id: Uuid,
    embedder_name: String,
    params: String,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.embedder = Some(Embedder {
                name: embedder_name,
                params: serde_json::Value::from(params),
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_vector_store(
    index_id: Uuid,
    vector_store_name: String,
    params: String,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.vector_store = Some(VectorStore {
                name: vector_store_name,
                params: serde_json::Value::from(params),
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_file_extractor(
    index_id: Uuid,
    file_extractor_name: String,
    params: String,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_extractor = Some(FileExtractor {
                name: file_extractor_name,
                params: serde_json::Value::from(params),
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn upsert_file_source(
    index_id: Uuid,
    file_source_name: String,
    oauth_id: Option<Uuid>,
    params: String,
) -> Result<(), String> {
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_source = Some(models::FileSource {
                name: file_source_name,
                params: serde_json::Value::from(params),
                oauth_id: oauth_id,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn test_file_source(
    file_source_name: String,
    params_json: String,
    oauth_id: Option<Uuid>,
) -> Result<(), String> {
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    let userdata = load()?;
    let oauth = match oauth_id {
        Some(id) => userdata.oauth_clients.iter().find(|v| v.id == id),
        None => None,
    };
    let auth_result = match oauth.as_ref() {
        Some(model) => Some(model.do_oauth_pkce().await.map_err(|err| err.to_string())?),
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
        let _ = model
            .revoke_access_token(auth_result.access_token.secret())
            .await;
        if let Some(refresh_token) = auth_result.refresh_token.as_ref() {
            let _ = model.revoke_refresh_token(refresh_token.secret()).await;
        }
    }
    result
}
