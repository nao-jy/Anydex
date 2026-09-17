use std::ffi::{c_char, CStr, CString};

use libloading::Library;

use crate::plugin::manager::LibraryLoader;

type Handler = u64;

pub trait Embedder {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(&self, param: &serde_json::Value) -> Handler;
    fn free_handler(&self, handler: Handler);
    // `param` must match the schema returned by `get_param_schema`.
    fn embed(&self, handler: Handler, file_path: &str) -> Vec<f32>;
}

/// Host-side adapter for an embedder supplied by a dynamic library.
///
/// A library must export these C ABI symbols:
/// `anydex_embedder_get_param_schema`,
/// `anydex_embedder_get_handler`, `anydex_embedder_free_handler`,
/// `anydex_embedder_embed`, `anydex_embedder_free_string`, and
/// `anydex_embedder_free_embedding`.
pub struct DynamicEmbedder {
    get_param_schema: GetParamSchema,
    get_handler: GetHandler,
    free_handler: FreeHandler,
    embed: Embed,
    free_string: FreeString,
    free_embedding: FreeEmbedding,
}

type GetParamSchema = unsafe extern "C" fn() -> *mut c_char;
type GetHandler = unsafe extern "C" fn(params: *const c_char) -> Handler;
type FreeHandler = unsafe extern "C" fn(handler: Handler);
type Embed = unsafe extern "C" fn(
    handler: Handler,
    file_path: *const c_char,
    length: *mut usize,
) -> *mut f32;
type FreeString = unsafe extern "C" fn(value: *mut c_char);
type FreeEmbedding = unsafe extern "C" fn(values: *mut f32, length: usize);

impl LibraryLoader for DynamicEmbedder {
    fn load_library(library: &Library) -> Result<Self, String> {
        // Function pointers are copied out of `Symbol`. `Plugin` owns `library`,
        // so those pointers remain valid for as long as this adapter is alive.
        unsafe {
            Ok(Self {
                get_param_schema: *library.get(b"anydex_embedder_get_param_schema\0").map_err(
                    |error| format!("missing anydex_embedder_get_param_schema: {error}"),
                )?,
                get_handler: *library
                    .get(b"anydex_embedder_get_handler\0")
                    .map_err(|error| format!("missing anydex_embedder_get_handler: {error}"))?,
                free_handler: *library
                    .get(b"anydex_embedder_free_handler\0")
                    .map_err(|error| format!("missing anydex_embedder_free_handler: {error}"))?,
                embed: *library
                    .get(b"anydex_embedder_embed\0")
                    .map_err(|error| format!("missing anydex_embedder_embed: {error}"))?,
                free_string: *library
                    .get(b"anydex_embedder_free_string\0")
                    .map_err(|error| format!("missing anydex_embedder_free_string: {error}"))?,
                free_embedding: *library
                    .get(b"anydex_embedder_free_embedding\0")
                    .map_err(|error| format!("missing anydex_embedder_free_embedding: {error}"))?,
            })
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
        unsafe { (self.free_string)(raw_schema) };
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
        unsafe { (self.free_embedding)(raw_embedding, length) };
        embedding
    }
}
