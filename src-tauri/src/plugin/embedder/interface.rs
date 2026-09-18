use std::ffi::{c_char, c_void, CStr, CString};

use libloading::Library;

use crate::plugin::{
    interface::{get_plugin, DynamicPluginHeader},
    manager::LibraryLoader,
};

type Handler = u64;

/// ABI version implemented by this host.
pub const DYNAMIC_EMBEDDER_ABI_VERSION_V1: i32 = 1;

pub trait Embedder {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(&self, param: &serde_json::Value) -> Handler;
    fn free_handler(&self, handler: Handler);
    // `param` must match the schema returned by `get_param_schema`.
    fn embed(&self, handler: Handler, file_path: &str) -> Vec<f32>;
}

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
///
/// The first two fields are a stable header. Future ABI revisions may only append
/// fields; they must not reorder or remove existing fields. Every pointer returned
/// by `get_plugin` must point to at least `struct_size` bytes and remain valid for
/// as long as the dynamic library is loaded.
#[repr(C)]
pub struct DynamicEmbedderApi {
    /// Must equal [`DYNAMIC_EMBEDDER_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema: Option<unsafe extern "C" fn() -> *mut c_char>,
    pub get_handler: Option<unsafe extern "C" fn(params: *const c_char) -> Handler>,
    pub free_handler: Option<unsafe extern "C" fn(handler: Handler)>,
    pub embed: Option<
        unsafe extern "C" fn(
            handler: Handler,
            file_path: *const c_char,
            length: *mut usize,
        ) -> *mut f32,
    >,
    /// Releases allocations returned by this plugin's ABI functions.
    pub free: Option<unsafe extern "C" fn(value: *mut c_void)>,
}

/// Validated host-side adapter for an embedder supplied by a dynamic library.
pub struct DynamicEmbedder {
    get_param_schema: unsafe extern "C" fn() -> *mut c_char,
    get_handler: unsafe extern "C" fn(params: *const c_char) -> Handler,
    free_handler: unsafe extern "C" fn(handler: Handler),
    embed: unsafe extern "C" fn(
        handler: Handler,
        file_path: *const c_char,
        length: *mut usize,
    ) -> *mut f32,
    free: unsafe extern "C" fn(value: *mut c_void),
}

impl LibraryLoader for DynamicEmbedder {
    fn load_library(library: &Library) -> Result<Self, String> {
        let plugin: *const DynamicEmbedderApi = get_plugin(library)?;
        let header = unsafe { std::ptr::read(plugin.cast::<DynamicPluginHeader>()) };

        if header.abi_version != DYNAMIC_EMBEDDER_ABI_VERSION_V1 {
            return Err(format!(
                "unsupported embedder plugin version {}; host supports {}",
                header.abi_version, DYNAMIC_EMBEDDER_ABI_VERSION_V1
            ));
        }
        let required_size = std::mem::size_of::<DynamicEmbedderApi>();
        if header.struct_size < required_size {
            return Err(format!(
                "embedder API table is too small: got {}, need at least {}",
                header.struct_size, required_size
            ));
        }

        let api = unsafe { std::ptr::read(plugin) };

        if api.abi_version == DYNAMIC_EMBEDDER_ABI_VERSION_V1 {
            Ok(Self {
                get_param_schema: api
                    .get_param_schema
                    .ok_or("embedder API has a null get_param_schema pointer")?,
                get_handler: api
                    .get_handler
                    .ok_or("embedder API has a null get_handler pointer")?,
                free_handler: api
                    .free_handler
                    .ok_or("embedder API has a null free_handler pointer")?,
                embed: api.embed.ok_or("embedder API has a null embed pointer")?,
                free: api.free.ok_or("embedder API has a null free pointer")?,
            })
        } else {
            return Err(format!(
                "unsupported embedder plugin version {}",
                api.abi_version
            ));
        }
    }
}

impl Embedder for DynamicEmbedder {
    fn get_param_schema(&self) -> serde_json::Value {
        let raw_schema = unsafe { (self.get_param_schema)() };
        if raw_schema.is_null() {
            return serde_json::Value::Null;
        }

        let schema = unsafe { CStr::from_ptr(raw_schema).to_string_lossy().into_owned() };
        unsafe { (self.free)(raw_schema as *mut c_void) };
        serde_json::from_str(&schema).expect("embedder parameter schema must be valid JSON")
    }

    fn get_handler(&self, param: &serde_json::Value) -> Handler {
        let params = CString::new(
            serde_json::to_string(param).expect("embedder parameters must be serializable JSON"),
        )
        .expect("embedder parameters must not contain NUL bytes");
        unsafe { (self.get_handler)(params.as_ptr()) }
    }

    fn free_handler(&self, handler: Handler) {
        unsafe { (self.free_handler)(handler) }
    }

    fn embed(&self, handler: Handler, file_path: &str) -> Vec<f32> {
        let file_path =
            CString::new(file_path).expect("embedder file paths must not contain NUL bytes");
        let mut length = 0;
        let raw_embedding = unsafe { (self.embed)(handler, file_path.as_ptr(), &mut length) };

        if raw_embedding.is_null() {
            assert_eq!(
                length, 0,
                "embedder returned a null embedding with non-zero length"
            );
            return Vec::new();
        }

        let embedding = unsafe { std::slice::from_raw_parts(raw_embedding, length).to_vec() };
        unsafe { (self.free)(raw_embedding as *mut c_void) };
        embedding
    }
}
