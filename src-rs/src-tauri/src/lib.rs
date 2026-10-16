// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use specta_typescript::Typescript;
use tauri::{generate_handler, Manager};
use tauri_specta::{collect_commands, collect_events, Builder};
use tracing::error;

use crate::{
    path::init,
    plugin::entry::PluginType,
    window::task_tray::{init_task_tray, prevent_close},
};

pub mod bg_job;
pub mod consts;
pub mod dto;
pub mod ipc;
pub mod logging;
pub mod path;
pub mod plugin;
pub mod repository;
pub mod server;
pub mod usecase;
pub mod window;

macro_rules! app_commands {
    ($command_macro:ident) => {
        $command_macro![
            ipc::indexes::list_indexes,
            ipc::indexes::create_empty_index,
            ipc::indexes::rename_index,
            ipc::indexes::put_description,
            ipc::indexes::set_index_mcp_server,
            ipc::indexes::delete_index,
            ipc::indexes::clear_index,
            ipc::indexes::upsert_schedule,
            ipc::indexes::start_indexing,
            ipc::mcp::get_mcp_server_state,
            ipc::mcp::set_mcp_server_status,
            ipc::jobs::list_active_jobs,
            ipc::settings::get_application_settings,
            ipc::plugins::get_plugin_info,
            ipc::plugins::upsert_embedder,
            ipc::plugins::upsert_vector_store,
            ipc::plugins::upsert_file_extractor,
            ipc::plugins::upsert_file_source,
            ipc::plugins::test_file_source,
            ipc::plugins::test_embedder,
            ipc::plugins::get_param_schema,
            ipc::search::list_directory,
            ipc::search::grep,
            ipc::search::semantic_search,
            ipc::search::get_file_info,
            ipc::search::download_file,
            ipc::oauth::list_oauth_client,
            ipc::oauth::create_empty_oauth_client,
            ipc::oauth::update_oauth_client,
            ipc::oauth::delete_oauth_client,
            ipc::oauth::test_oauth,
            ipc::oauth::abort_oauth,
            ipc::oauth::do_oauth_authentication_if_not,
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

            init(
                app.path().app_data_dir().unwrap().to_str().unwrap(),
                app.path().app_cache_dir().unwrap().to_str().unwrap(),
                dirs::download_dir().unwrap().to_str().unwrap(),
            )?;

            let database = tauri::async_runtime::block_on(repository::sqlite::init())?;
            app.manage(database.clone());

            tracing::info!("application database initialized");

            let task_tray_init = init_task_tray(app);
            if let Err(err) = task_tray_init {
                error!(err);
            } else {
                tracing::info!("task tray initialized");
            }

            let init_result = PluginType::init();
            if let Err(err) = init_result {
                error!(err);
            } else {
                tracing::info!("plugin initialized");
            }

            tauri::async_runtime::block_on(bg_job::entry::init(
                &database,
                &app.path().app_cache_dir().unwrap().to_string_lossy(),
            ))?;

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
