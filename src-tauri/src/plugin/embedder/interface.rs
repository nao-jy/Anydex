pub type Handler = u64;

pub trait Embedder {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(&self, param: &serde_json::Value) -> Result<Handler, String>;
    fn free_handler(&self, handler: Handler) -> Result<(), String>;
    // `param` must match the schema returned by `get_param_schema`.
    fn embed(&self, handler: Handler, file_path: &str) -> Result<Vec<f32>, String>;
}
