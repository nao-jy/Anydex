//! Application-wide tracing configuration.

use std::{error::Error, fs, path::PathBuf};
use tauri::{App, Manager, Runtime};
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

use crate::logging::layer_error_dialog::ErrorDialogLayer;

pub mod layer_error_dialog;

/// Keeps the non-blocking log writer alive until the application exits.
pub struct LogGuard {
    _worker: WorkerGuard,
}

/// Installs the global `tracing` subscriber.
///
/// Set `RUST_LOG` to change the filter, for example
/// `RUST_LOG=anydex_lib=debug,tao=warn,wry=warn`.
pub fn init<R: Runtime>(app: &App<R>) -> Result<(LogGuard, PathBuf), Box<dyn Error>> {
    let log_dir = app.path().app_log_dir()?;
    fs::create_dir_all(&log_dir)?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tao=warn,wry=warn"));
    let file_appender = rolling::daily(&log_dir, "anydex.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    let fmt_layer = fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(cfg!(debug_assertions))
        .compact()
        .with_filter(filter);

    let error_dlg_layer = ErrorDialogLayer {
        app: app.handle().clone(),
    };

    registry()
        .with(fmt_layer)
        .with(error_dlg_layer)
        .try_init()
        .map_err(std::io::Error::other)?;

    tracing::error!("TEST ERROR");

    Ok((LogGuard { _worker: guard }, log_dir))
}
