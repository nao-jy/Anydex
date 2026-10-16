use std::fs;

use enum_iterator::all;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    dto::models::PluginInfo,
    path,
    plugin::{
        embedder::interface::Embedder as EmbedderPlugin,
        entry::{PluginType, EMBEDDER_MGR, FILE_SOURCE_MGR, VECTOR_STORE_MGR},
        file_source::interface::FileSource,
        vector_store::interface::VectorStore,
    },
    repository::json::{
        load,
        models::{self, *},
        save,
    },
    usecase::oauth::get_authentication,
};

fn parse_plugin_params(params: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(params).map_err(|err| err.to_string())
}

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
    let params = parse_plugin_params(&params)?;
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.embedder = Some(Embedder {
                name: embedder_name,
                params,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn clear_vector_store(index_id: Uuid) -> Result<(), String> {
    let userdata = load()?;
    for index in &userdata.indexes {
        if index.id == index_id {
            if let Some(prev) = &index.vector_store {
                let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                let plugin_opt = mgr.get_plugin(&prev.name);
                if let Some(plugin) = plugin_opt {
                    if let Ok(handler) = plugin.get_handler(
                        &prev.params,
                        index_id.to_string(),
                        &path::get_dir().app_data_dir.to_string_lossy().to_string(),
                    ) {
                        let _ = plugin.clear_db(handler.handler);
                    }
                }
            }
            break;
        }
    }
    Ok(())
}

pub async fn upsert_vector_store(
    index_id: Uuid,
    vector_store_name: String,
    params: String,
) -> Result<(), String> {
    let params = parse_plugin_params(&params)?;
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            if let Some(prev) = &index.vector_store {
                let mut mgr_mtx = VECTOR_STORE_MGR.lock().unwrap();
                let mgr = mgr_mtx.as_mut().unwrap();
                let plugin_opt = mgr.get_plugin(&prev.name);
                if let Some(plugin) = plugin_opt {
                    if let Ok(handler) = plugin.get_handler(
                        &prev.params,
                        index_id.to_string(),
                        &path::get_dir().app_data_dir.to_string_lossy().to_string(),
                    ) {
                        let _ = plugin.clear_db(handler.handler);
                    }
                }
            }
            index.vector_store = Some(crate::usecase::plugins::models::VectorStore {
                name: vector_store_name,
                params,
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
    let params = parse_plugin_params(&params)?;
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_extractor = Some(FileExtractor {
                name: file_extractor_name,
                params,
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
    let params = parse_plugin_params(&params)?;
    let mut userdata = load()?;
    for index in userdata.indexes.iter_mut() {
        if index.id == index_id {
            index.file_source = Some(models::FileSource {
                name: file_source_name,
                params,
                oauth_id: oauth_id,
            });
            break;
        }
    }
    save(&userdata)
}

pub async fn test_file_source(
    index_id: Uuid,
    file_source_name: String,
    params_json: String,
    database: &DatabaseConnection,
) -> Result<(), String> {
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    let userdata = load()?;
    let oauth_id = userdata
        .indexes
        .iter()
        .find(|v| v.id == index_id)
        .ok_or("index not found")?
        .file_source
        .clone()
        .map(|v| v.oauth_id);
    let mut auth = match oauth_id {
        Some(Some(id)) => Some(
            get_authentication(index_id, id, database)
                .await
                .map_err(|err| err.to_string())?
                .ok_or("not authenticated yet")?,
        ),
        Some(None) => None,
        None => None,
    };
    let access_token = match auth.as_mut() {
        Some(v) => Some(v.load_access_token().await?),
        None => None,
    };
    let download_dir = std::env::temp_dir()
        .join(Uuid::new_v4().to_string())
        .to_string_lossy()
        .to_string();
    fs::create_dir_all(&download_dir).map_err(|err| err.to_string())?;
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
        (Ok(_), Err(err)) => Err(err),
        (Ok(()), Ok(())) => Ok(()),
    };

    result
}

pub async fn test_embedder(
    index_id: Uuid,
    embedder_name: String,
    params_json: String,
) -> Result<String, String> {
    let params = serde_json::from_str(&params_json).map_err(|err| err.to_string())?;
    let test_file = std::env::temp_dir().join(format!(
        "anydex-embedder-test-{index_id}-{}.txt",
        Uuid::new_v4()
    ));
    fs::write(
        &test_file,
        "Anydex embedder test. This text verifies that the configured model can generate an embedding.",
    )
    .map_err(|err| format!("failed to create embedder test input: {err}"))?;
    let test_file_path = test_file.to_string_lossy().into_owned();

    let plugin_result = tokio::task::spawn_blocking(move || {
        let mut manager = EMBEDDER_MGR.lock().unwrap();
        let manager = manager
            .as_mut()
            .ok_or_else(|| "embedder plugin manager is not initialized".to_owned())?;
        let plugin = manager.get_plugin(&embedder_name).ok_or_else(|| {
            format!("Embedder plugin {embedder_name} was not found in the plugin manager")
        })?;
        let handler = plugin.get_handler(&params).map_err(|err| err.to_string())?;
        let embeddings = plugin
            .embed(handler.handler, &test_file_path)
            .map_err(|err| err.to_string())?;
        let dimensions = params
            .get("dimensions")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| "embedder dimensions must be a positive integer".to_owned())?;
        if embeddings.len() != dimensions as usize {
            return Err(format!(
                "embedder returned {} dimensions, expected {dimensions}",
                embeddings.len()
            ));
        }
        if embeddings.iter().any(|value| !value.is_finite()) {
            return Err("embedder returned a non-finite vector value".to_owned());
        }
        Ok(format!(
            "Embedding test succeeded with {} dimensions",
            embeddings.len()
        ))
    })
    .await
    .map_err(|err| format!("embedder test task failed: {err}"))
    .and_then(|result| result);

    let cleanup_result = fs::remove_file(&test_file).map_err(|err| err.to_string());
    let result = match (plugin_result, cleanup_result) {
        (Err(err), Err(cleanup_err)) => {
            tracing::warn!("failed to remove embedder test input: {cleanup_err}");
            Err(err)
        }
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
        (Ok(message), Ok(())) => Ok(message),
    };

    result
}
