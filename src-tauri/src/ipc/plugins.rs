use sea_orm::{DatabaseConnection, EntityTrait};
use tauri::State;

use crate::{dto::models::PluginInfo, plugin::entry::PluginType, repository};

#[tauri::command]
#[specta::specta]
#[tracing::instrument()]
pub fn get_plugin_info() -> Result<Vec<PluginInfo>, String> {
    let ret = vec![
        PluginInfo {
            plugin_type: PluginType::FileSource.into(),
            path: PluginType::FileSource.get_dir()?,
            plugins: PluginType::FileSource.list_names(),
        },
        PluginInfo {
            plugin_type: PluginType::FileExtractor.into(),
            path: PluginType::FileExtractor.get_dir()?,
            plugins: PluginType::FileExtractor.list_names(),
        },
        PluginInfo {
            plugin_type: PluginType::Embedder.into(),
            path: PluginType::Embedder.get_dir()?,
            plugins: PluginType::Embedder.list_names(),
        },
        PluginInfo {
            plugin_type: PluginType::VectorStore.into(),
            path: PluginType::VectorStore.get_dir()?,
            plugins: PluginType::VectorStore.list_names(),
        },
    ];
    Ok(ret)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(database))]
pub async fn upsert_embedder(
    index_id: i32,
    embedder_name: String,
    dimensions: i32,
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

    repository::indexes::upsert_embedder(&database, index_id, embedder_name, dimensions, params)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}
