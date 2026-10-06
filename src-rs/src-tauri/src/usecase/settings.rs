use std::path::PathBuf;

use crate::dto::models::ApplicationSettings;

pub fn get_application_settings(
    app_data_dir: Option<PathBuf>,
    app_log_dir: Option<PathBuf>,
) -> ApplicationSettings {
    let app_data_dir = app_data_dir.unwrap_or_else(|| PathBuf::from("."));
    let log_directory = app_log_dir.unwrap_or_else(|| app_data_dir.join("logs"));

    ApplicationSettings {
        database_path: app_data_dir.join("anydex.sqlite3").display().to_string(),
        log_directory: log_directory.display().to_string(),
    }
}
