use std::ffi::{c_char, c_void, CStr, CString};

use libloading::Library;

use crate::plugin::{
    embedder::interface::{Embedder, Handler},
    interface::{get_plugin, DynamicPluginHeader},
    manager::LibraryLoader,
};

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
    pub embed: unsafe extern "C" fn(
        handler: Handler,
        file_path: *const c_char,
        length: *mut usize,
    ) -> *const f32,
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
    fn get_param_schema(&self) -> serde_json::Value {
        match self {
            DynamicEmbedder::V1(embedder_v1) => {
                let raw_schema = unsafe { (embedder_v1.get_param_schema)() };
                if raw_schema.is_null() {
                    return serde_json::Value::Null;
                }

                let schema = unsafe { CStr::from_ptr(raw_schema).to_string_lossy().into_owned() };
                unsafe { (embedder_v1.free)(raw_schema as *mut c_void) };
                serde_json::from_str(&schema).expect("embedder parameter schema must be valid JSON")
            }
        }
    }

    fn get_handler(&self, param: &serde_json::Value) -> Handler {
        match self {
            DynamicEmbedder::V1(embedder_v1) => {
                let params = CString::new(
                    serde_json::to_string(param)
                        .expect("embedder parameters must be serializable JSON"),
                )
                .expect("embedder parameters must not contain NUL bytes");
                unsafe { (embedder_v1.get_handler)(params.as_ptr()) }
            }
        }
    }

    fn free_handler(&self, handler: Handler) {
        match self {
            DynamicEmbedder::V1(embedder_v1) => unsafe { (embedder_v1.free_handler)(handler) },
        }
    }

    fn embed(&self, handler: Handler, file_path: &str) -> Vec<f32> {
        match self {
            DynamicEmbedder::V1(embedder_v1) => {
                let file_path = CString::new(file_path)
                    .expect("embedder file paths must not contain NUL bytes");
                let mut length = 0;
                let raw_embedding =
                    unsafe { (embedder_v1.embed)(handler, file_path.as_ptr(), &mut length) };

                if raw_embedding.is_null() {
                    assert_eq!(
                        length, 0,
                        "embedder returned a null embedding with non-zero length"
                    );
                    return Vec::new();
                }

                let embedding =
                    unsafe { std::slice::from_raw_parts(raw_embedding, length).to_vec() };
                unsafe { (embedder_v1.free)(raw_embedding as *mut c_void) };
                embedding
            }
        }
    }
}
