use std::ffi::{c_void, CString};

use anydex_plugin_api_lib::{
    common::{to_string, Handler},
    embedder::interface::{DynamicEmbedder, DynamicEmbedderV1, DYNAMIC_EMBEDDER_ABI_VERSION_V1},
    interface::DynamicPluginHeader,
};
use libloading::Library;

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

    fn get_handler(&self, param: &serde_json::Value) -> Result<DisposableHnandler, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                let handler = unsafe { (plugin_v1.get_handler)(params.as_ptr()) };
                let free_handler = plugin_v1.free_handler;
                Ok(DisposableHnandler {
                    handler,
                    free_fn: Box::new(move |h| unsafe { free_handler(h) }),
                })
            }
        }
    }

    fn embed(&self, handler: Handler, file_path: &str) -> Result<Vec<f32>, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let file_path = CString::new(file_path).map_err(|err| err.to_string())?;
                let raw_embedding = unsafe { (plugin_v1.embed)(handler, file_path.as_ptr()) };
                let embedding = unsafe { raw_embedding.read() };

                if embedding.ptr.is_null() {
                    return Err("embed returned a null".into());
                }

                let embedding = embedding.into();
                unsafe { (plugin_v1.free)(raw_embedding as *const c_void) };
                Ok(embedding)
            }
        }
    }
}
