use chrono::Utc;
use std::path::PathBuf;
use tauri::Manager;

use crate::dto::models::{ApplicationSettings, PluginDirectory, PluginReloadResult};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(app))]
pub fn get_application_settings(app: tauri::AppHandle) -> ApplicationSettings {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let log_directory = app
        .path()
        .app_log_dir()
        .unwrap_or_else(|_| app_data_dir.join("logs"));

    ApplicationSettings {
        database_path: app_data_dir.join("anydex.sqlite3").display().to_string(),
        log_directory: log_directory.display().to_string(),
        plugin_directories: vec![
            PluginDirectory {
                name: "ファイルソース".into(),
                path: app_data_dir
                    .join("plugins/file-sources")
                    .display()
                    .to_string(),
                plugin_count: 1,
            },
            PluginDirectory {
                name: "抽出器".into(),
                path: app_data_dir
                    .join("plugins/extractors")
                    .display()
                    .to_string(),
                plugin_count: 2,
            },
            PluginDirectory {
                name: "ベクトルストア".into(),
                path: app_data_dir
                    .join("plugins/vector-stores")
                    .display()
                    .to_string(),
                plugin_count: 1,
            },
            PluginDirectory {
                name: "ベクトル化".into(),
                path: app_data_dir.join("plugins/embedders").display().to_string(),
                plugin_count: 1,
            },
        ],
    }
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument]
pub fn reload_plugins() -> PluginReloadResult {
    PluginReloadResult {
        reloaded_at: Utc::now(),
        discovered_plugins: 4,
    }
}
