use std::path::PathBuf;
use tauri::Manager;

use crate::dto::models::ApplicationSettings;

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
    }
}
