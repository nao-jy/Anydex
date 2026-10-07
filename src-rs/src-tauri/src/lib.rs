// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use specta_typescript::Typescript;
use tauri::{generate_handler, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};
use tracing::error;

use crate::{
    plugin::entry::PluginType,
    window::task_tray::{init_task_tray, prevent_close},
};

pub mod consts;
pub mod dto;
pub mod ipc;
pub mod logging;
pub mod plugin;
pub mod repository;
pub mod server;
pub mod usecase;
pub mod window;

macro_rules! app_commands {
    ($command_macro:ident) => {
        $command_macro![
            ipc::search::greet,
            ipc::indexes::list_indexes,
            ipc::indexes::create_empty_index,
            ipc::indexes::rename_index,
            ipc::indexes::put_description,
            ipc::indexes::set_index_mcp_server,
            ipc::indexes::delete_index,
            ipc::indexes::upsert_schedule,
            ipc::mcp::get_mcp_server_state,
            ipc::mcp::set_mcp_server_status,
            ipc::tasks::list_active_tasks,
            ipc::settings::get_application_settings,
            ipc::plugins::get_plugin_info,
            ipc::plugins::upsert_embedder,
            ipc::plugins::upsert_vector_store,
            ipc::plugins::upsert_file_extractor,
            ipc::plugins::upsert_file_source,
            ipc::plugins::test_file_source,
            ipc::plugins::reload_plugins,
            ipc::plugins::get_param_schema,
            ipc::search::search_mock,
            ipc::oauth::list_oauth_client,
            ipc::oauth::create_empty_oauth_client,
            ipc::oauth::update_oauth_client,
            ipc::oauth::delete_oauth_client,
            ipc::oauth::test_oauth,
            ipc::oauth::abort_oauth,
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let exporter: Builder<tauri::Wry> = Builder::new()
        .commands(app_commands!(collect_commands))
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    exporter
        .export(Typescript::default(), "../../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        use crate::window::singleton::on_second_instance;

        builder = builder.plugin(tauri_plugin_single_instance::init(on_second_instance))
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let (log_guard, log_dir) = logging::init(app)?;
            app.manage(log_guard);
            tracing::info!(path = %log_dir.display(), "file logging initialized");

            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("anydex.sqlite3");
            let database_url = repository::sqlite::database_url(&database_path)?;
            let database =
                tauri::async_runtime::block_on(repository::sqlite::initialize(&database_url))?;
            app.manage(database);

            let json_path = app_data_dir.join("userdata.json");
            repository::json::initialize(&json_path.to_string_lossy().to_string())?;

            tracing::info!(path = %database_path.display(), "application database initialized");

            let task_tray_init = init_task_tray(app);
            if let Err(err) = task_tray_init {
                error!(err);
            } else {
                tracing::info!("task tray initialized");
            }

            let init_result = PluginType::init(app_data_dir.to_str().unwrap());
            if let Err(err) = init_result {
                error!(err);
            } else {
                tracing::info!(path = %database_path.display(), "plugin initialized");
            }

            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(app_commands!(generate_handler))
        .on_window_event(prevent_close)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_typescript_bindings() {
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/bindings.ts");
        let builder: Builder<tauri::Wry> = Builder::new()
            .commands(app_commands!(collect_commands))
            .events(collect_events![]);

        builder
            .export(Typescript::default(), &output)
            .expect("DTOs must export to TypeScript");
    }
}
