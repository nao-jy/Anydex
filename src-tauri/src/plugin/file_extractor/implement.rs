use crate::plugin::{
    common::{get_plugin, DisposableHnandler},
    file_extractor::interface::{ExtractedFileRelativePath, FileExtractor},
    manager::LibraryLoader,
};
use anydex_plugin_api_lib::{
    common::{to_string, Handler},
    file_extractor::interface::{
        DynamicFileExtractor, DynamicFileExtractorV1, DYNAMIC_FILE_EXTRACTOR_ABI_VERSION_V1,
    },
    interface::DynamicPluginHeader,
};
use libloading::Library;
use std::ffi::{c_char, c_void, CString};

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
                let raw_schema = unsafe { (plugin_v1.get_param_schema)() };
                if raw_schema.is_null() {
                    return Err("get_param_schema returns null".into());
                }

                let schema = to_string(raw_schema);
                unsafe { (plugin_v1.free)(raw_schema as *mut c_void) };
                serde_json::from_str(&schema).map_err(|err| err.to_string())
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
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                let base_dir = CString::new(base_dir).map_err(|err| err.to_string())?;
                let handler =
                    unsafe { (plugin_v1.get_handler)(params.as_ptr(), base_dir.as_ptr()) };
                let free_handler = plugin_v1.free_handler;
                Ok(DisposableHnandler {
                    handler,
                    free_fn: Box::new(move |h| unsafe { free_handler(h) }),
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
                let file_abs_path = CString::new(file_abs_path).map_err(|err| err.to_string())?;
                let extracted_ptr = unsafe { (plugin_v1.extract)(handler, file_abs_path.as_ptr()) };
                let extracted = unsafe { extracted_ptr.read() };
                let path_vec: Vec<*const c_char> = extracted.into();

                let ret: Vec<String> = path_vec.iter().map(|&c| to_string(c)).collect();

                unsafe { (plugin_v1.free)(extracted_ptr as *const c_void) };
                Ok(ret)
            }
        }
    }
}
