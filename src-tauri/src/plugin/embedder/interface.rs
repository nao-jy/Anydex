use anydex_plugin_api_lib::common::Handler;

use crate::plugin::common::DisposableHnandler;

pub trait Embedder {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(&self, param: &serde_json::Value) -> Result<DisposableHnandler, String>;
    // `param` must match the schema returned by `get_param_schema`.
    fn embed(&self, handler: Handler, file_path: &str) -> Result<Vec<f32>, String>;
}
