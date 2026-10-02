use std::ffi::c_char;

use anydex_plugin_api_lib::{
    common::{to_string, Handler, LogLevel},
    interface::{GetPlugin, PluginInit},
};
use libloading::{Library, Symbol};
use tracing::{debug, error, info, trace, warn};

pub unsafe extern "C" fn host_log(log_level: LogLevel, message: *const c_char) {
    let message = to_string(message);
    match log_level {
        LogLevel::Error => error!("{}", message),
        LogLevel::Warn => warn!("{}", message),
        LogLevel::Info => info!("{}", message),
        LogLevel::Debug => debug!("{}", message),
        LogLevel::Trace => trace!("{}", message),
    }
}

pub struct DisposableHnandler {
    pub handler: Handler,
    pub free_fn: Box<dyn FnOnce(Handler)>,
}

impl Drop for DisposableHnandler {
    fn drop(&mut self) {
        let free_fn = std::mem::replace(&mut self.free_fn, Box::new(|_| {}));
        free_fn(self.handler);
    }
}

// Get plugin pointer
pub fn get_plugin<T>(library: &Library) -> Result<*const T, String> {
    let get_plugin: Symbol<GetPlugin<T>> = unsafe {
        library
            .get(b"get_plugin")
            .map_err(|error| format!("missing get_plugin: {error}"))?
    };
    let init = PluginInit {
        abi_version: 1,
        struct_size: std::mem::size_of::<PluginInit>(),
        log: host_log,
    };
    let plugin = unsafe { get_plugin(&init) };
    if plugin.is_null() {
        return Err("get_plugin returned a null pointer".to_string());
    }
    Ok(plugin)
}
