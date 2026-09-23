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
