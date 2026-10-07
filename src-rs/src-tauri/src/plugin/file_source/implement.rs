use anydex_plugin_api_lib::{
    common::{Handler, PluginFn},
    file_source::{interface::*, params::*, types::*},
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

    fn get_handler(&self, param: &serde_json::Value) -> Result<DisposableHnandler, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
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

    fn get_diff(
        &self,
        handler: Handler,
        last_indexed_files: &[InputFileInfo],
        access_token: Option<String>,
    ) -> Result<GetDiffData, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let args = GetDiffArgs {
                    handler: handler,
                    last_indexed_files: last_indexed_files.to_vec(),
                    access_token,
                };
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result);
            }
        }
    }

    fn download(
        &self,
        handler: Handler,
        uri: &str,
        access_token: Option<String>,
        dst_dir: &str,
    ) -> Result<String, String> {
        match self {
            DynamicFileSource::V1(plugin_v1) => {
                let args = DownloadArgs {
                    handler: handler,
                    dst_dir: dst_dir.to_string(),
                    uri: uri.to_string(),
                    access_token,
                };
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.downloaded_file_path);
            }
        }
    }
}
