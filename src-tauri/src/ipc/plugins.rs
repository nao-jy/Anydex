use std::path::PathBuf;
use tauri::Manager;

use crate::dto::models::PluginInfo;

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
            plugin_type: "file-sources".into(),
            path: app_data_dir
                .join("plugins")
                .join("file-sources")
                .display()
                .to_string(),
            plugins: vec!["OpenDAL".into()],
        },
        PluginInfo {
            plugin_type: "extractors".into(),
            path: app_data_dir
                .join("plugins")
                .join("extractors")
                .display()
                .to_string(),
            plugins: vec!["Default".into()],
        },
        PluginInfo {
            plugin_type: "vector-stores".into(),
            path: app_data_dir
                .join("plugins")
                .join("vector-stores")
                .display()
                .to_string(),
            plugins: vec!["sqlite-vector".into()],
        },
        PluginInfo {
            plugin_type: "embedders".into(),
            path: app_data_dir
                .join("plugins")
                .join("embedders")
                .display()
                .to_string(),
            plugins: vec!["JSON Web API".into()],
        },
    ]
}
