use anydex_plugin_api_lib::{
    common::{Handler, PluginFn},
    file_source::{interface::*, params::*},
    interface::DynamicPluginHeader,
};
use libloading::Library;
use tracing::error;

use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    file_source::interface::FileSource,
    manager::LibraryLoader,
};

impl LibraryLoader for DynamicFileSource {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin_header_ptr: *const DynamicPluginHeader = get_plugin(library)?;
        let plugin_header = unsafe { std::ptr::read(plugin_header_ptr) };

        match plugin_header.abi_version {
            DYNAMIC_FILE_SOURCE_ABI_VERSION_V1 => {
                let required_size = std::mem::size_of::<DynamicFileSourceV1>();
                if plugin_header.struct_size != required_size {
                    return Err(format!(
                        "embedder API table is not same as expected: got {}, need {}",
                        plugin_header.struct_size, required_size
                    ));
                }
                let plugin_api =
                    unsafe { std::ptr::read(plugin_header_ptr.cast::<DynamicFileSourceV1>()) };
                Ok(DynamicFileSource::V1(plugin_api))
            }
            version => Err(format!("unsupported version: {version}")),
        }
    }
}

impl FileSource for DynamicFileSource {
    fn get_param_schema(&self) -> Result<serde_json::Value, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let args = GetParamSchemaArgs {};
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.schema);
            }
        }
    }

    fn get_handler(
        &self,
        param: &serde_json::Value,
        last_indexed_files: &[InputFileInfo],
        access_token: Option<String>,
        download_dir: &str,
    ) -> Result<DisposableHnandler, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let args = GetHandlerArgs {
                    params: param.clone(),
                    access_token: access_token,
                    last_indexed_files: last_indexed_files.to_vec(),
                    download_dir: download_dir.to_string(),
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

    fn next(&self, handler: Handler) -> Result<Option<FileIterationInfo>, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let args = NextArgs { handler: handler };
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.info);
            }
        }
    }
}
