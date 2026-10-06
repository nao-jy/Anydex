use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    file_extractor::interface::{ExtractedFileRelativePath, FileExtractor},
    manager::LibraryLoader,
};
use anydex_plugin_api_lib::{
    common::{Handler, PluginFn},
    file_extractor::{interface::*, params::*},
    interface::DynamicPluginHeader,
};
use libloading::Library;
use tracing::error;

impl LibraryLoader for DynamicFileExtractor {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin_header_ptr: *const DynamicPluginHeader = get_plugin(library)?;
        let plugin_header = unsafe { std::ptr::read(plugin_header_ptr) };

        match plugin_header.abi_version {
            DYNAMIC_FILE_EXTRACTOR_ABI_VERSION_V1 => {
                let required_size = std::mem::size_of::<DynamicFileExtractorV1>();
                if plugin_header.struct_size != required_size {
                    return Err(format!(
                        "embedder API table is not same as expected: got {}, need {}",
                        plugin_header.struct_size, required_size
                    ));
                }
                let plugin_api =
                    unsafe { std::ptr::read(plugin_header_ptr.cast::<DynamicFileExtractorV1>()) };
                Ok(DynamicFileExtractor::V1(plugin_api))
            }
            version => Err(format!("unsupported version: {version}")),
        }
    }
}

impl FileExtractor for DynamicFileExtractor {
    fn get_param_schema(&self) -> Result<serde_json::Value, String> {
        match self {
            DynamicFileExtractor::V1(plugin_v1) => {
                let args = GetParamSchemaArgs {};
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.schema);
            }
        }
    }

    fn get_handler(
        &self,
        param: &serde_json::Value,
        base_dir: &str,
    ) -> Result<DisposableHnandler, String> {
        match self {
            DynamicFileExtractor::V1(plugin_v1) => {
                let args = GetHandlerArgs {
                    params: param.clone(),
                    base_dir: base_dir.to_string(),
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

    fn extract(
        &self,
        handler: Handler,
        file_abs_path: &str,
    ) -> Result<Vec<ExtractedFileRelativePath>, String> {
        match self {
            DynamicFileExtractor::V1(plugin_v1) => {
                let args = ExtractArgs {
                    handler: handler,
                    file_path: file_abs_path.to_string(),
                };
                let result = plugin_v1.apply_and_unwrap(args)?;
                return Ok(result.extracted_file_paths);
            }
        }
    }
}
