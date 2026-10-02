use anydex_plugin_api_lib::common::Handler;

use crate::plugin::common::DisposableHnandler;

pub type ExtractedFileRelativePath = String;

pub trait FileExtractor {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(
        &self,
        param: &serde_json::Value,
        base_dir: &str,
    ) -> Result<DisposableHnandler, String>;

    fn extract(
        &self,
        handler: Handler,
        file_abs_path: &str,
    ) -> Result<Vec<ExtractedFileRelativePath>, String>;
}
