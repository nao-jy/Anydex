pub type Handler = u64;

pub trait Embedder {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(&self, param: &serde_json::Value) -> Handler;
    fn free_handler(&self, handler: Handler);
    // `param` must match the schema returned by `get_param_schema`.
    fn embed(&self, handler: Handler, file_path: &str) -> Vec<f32>;
}
