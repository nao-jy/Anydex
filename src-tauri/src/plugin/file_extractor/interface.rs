pub type Handler = u64;
pub type ExtractedFileRelativePath = String;

pub trait FileExtractor {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(&self, param: &serde_json::Value, base_dir: &str) -> Result<Handler, String>;
    fn free_handler(&self, handler: Handler) -> Result<(), String>;

    fn extract(
        &self,
        handler: Handler,
        file_abs_path: &str,
    ) -> Result<Vec<ExtractedFileRelativePath>, String>;
}
