use std::path::PathBuf;
use tauri::Manager;

use crate::{dto::models::PluginInfo, plugin::def::PluginType};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(app))]
pub fn get_plugin_info(app: tauri::AppHandle) -> Vec<PluginInfo> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    vec![
        PluginInfo {
            plugin_type: PluginType::FileSource.into(),
            path: PluginType::FileSource.get_dir(&app_data_dir),
            plugins: PluginType::FileSource.list_names(&app_data_dir),
        },
        PluginInfo {
            plugin_type: PluginType::FileExtractor.into(),
            path: PluginType::FileExtractor.get_dir(&app_data_dir),
            plugins: PluginType::FileExtractor.list_names(&app_data_dir),
        },
        PluginInfo {
            plugin_type: PluginType::Embedder.into(),
            path: PluginType::Embedder.get_dir(&app_data_dir),
            plugins: PluginType::Embedder.list_names(&app_data_dir),
        },
        PluginInfo {
            plugin_type: PluginType::VectorStore.into(),
            path: PluginType::VectorStore.get_dir(&app_data_dir),
            plugins: PluginType::VectorStore.list_names(&app_data_dir),
        },
    ]
}
