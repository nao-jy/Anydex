use tauri::Manager;

use crate::{dto::models::ApplicationSettings, usecase};

#[tauri::command]
#[specta::specta]
#[tracing::instrument(skip(app))]
pub fn get_application_settings(app: tauri::AppHandle) -> ApplicationSettings {
    let app_data_dir = app.path().app_data_dir().ok();
    let app_log_dir = app.path().app_log_dir().ok();
    usecase::settings::get_application_settings(app_data_dir, app_log_dir)
}
