use anydex_plugin_api_lib::{
    common::{Handler, PluginFn},
    embedder::{interface::*, params::*},
    interface::DynamicPluginHeader,
};
use libloading::Library;
use tracing::error;

use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    embedder::interface::Embedder,
    manager::LibraryLoader,
};

impl LibraryLoader for DynamicEmbedder {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin_header_ptr: *const DynamicPluginHeader = get_plugin(library)?;
        let plugin_header = unsafe { std::ptr::read(plugin_header_ptr) };

        match plugin_header.abi_version {
            DYNAMIC_EMBEDDER_ABI_VERSION_V1 => {
                let required_size = std::mem::size_of::<DynamicEmbedderV1>();
                if plugin_header.struct_size != required_size {
                    return Err(format!(
                        "embedder API table is not same as expected: got {}, need {}",
                        plugin_header.struct_size, required_size
                    ));
                }
                let plugin_api =
                    unsafe { std::ptr::read(plugin_header_ptr.cast::<DynamicEmbedderV1>()) };
                Ok(DynamicEmbedder::V1(plugin_api))
            }
            version => Err(format!("unsupported version: {version}")),
        }
    }
}

impl Embedder for DynamicEmbedder {
    fn get_param_schema(&self) -> Result<serde_json::Value, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let args = GetParamSchemaArgs {};
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.schema);
            }
        }
    }

    fn get_handler(&self, param: &serde_json::Value) -> Result<DisposableHnandler, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let args = GetHandlerArgs {
                    params: param.clone(),
                };
                let result = plugin_v1.apply_and_unwrap(args)?;

                let plugin_v1 = *plugin_v1;
                let free_handler = move |h| {
                    let args = FreeHandlerArgs { handler: h };
                    let result = plugin_v1.apply_and_unwrap(args);
                    if let Err(err) = result {
                        error!("{err}");
                    }
                };
                Ok(DisposableHnandler {
                    handler: result.handler,
                    free_fn: Box::new(free_handler),
                })
            }
        }
    }

    fn embed(&self, handler: Handler, file_path: &str) -> Result<Vec<f32>, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let args = EmbedArgs {
                    handler: handler,
                    file_path: file_path.to_string(),
                };
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.embeddings);
            }
        }
    }
}
