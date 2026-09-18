type Handler = u64;
type ExtractedFilePath = String;

pub trait FileExtractor {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(&self, param: &serde_json::Value, base_dir: &str) -> Handler;
    fn free_handler(&self, handler: Handler);

    fn extract(&self, handler: Handler, file_path: &str) -> Result<Vec<ExtractedFilePath>, String>;
}
