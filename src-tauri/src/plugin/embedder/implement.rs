use crate::plugin::{
    common::{to_string, CSlice},
    embedder::interface::{Embedder, Handler},
    interface::{get_plugin, DynamicPluginHeader},
    manager::LibraryLoader,
};
use libloading::Library;
use std::ffi::{c_char, c_void, CString};

/// ABI version implemented by this host.
pub const DYNAMIC_EMBEDDER_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
pub struct DynamicEmbedderV1 {
    /// Must equal [`DYNAMIC_EMBEDDER_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema: unsafe extern "C" fn() -> *const c_char,
    pub get_handler: unsafe extern "C" fn(params: *const c_char) -> Handler,
    pub free_handler: unsafe extern "C" fn(handler: Handler),
    pub embed:
        unsafe extern "C" fn(handler: Handler, file_path: *const c_char) -> *const CSlice<f32>,
    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: *const c_void),
}

pub enum DynamicEmbedder {
    V1(DynamicEmbedderV1),
}

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

    fn get_handler(&self, param: &serde_json::Value) -> Result<Handler, String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => {
                let params =
                    CString::new(serde_json::to_string(param).map_err(|err| err.to_string())?)
                        .map_err(|err| err.to_string())?;
                Ok(unsafe { (plugin_v1.get_handler)(params.as_ptr()) })
            }
        }
    }

    fn free_handler(&self, handler: Handler) -> Result<(), String> {
        match self {
            DynamicEmbedder::V1(plugin_v1) => Ok(unsafe { (plugin_v1.free_handler)(handler) }),
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
