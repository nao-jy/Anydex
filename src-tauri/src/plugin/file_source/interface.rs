use anydex_plugin_api_lib::{
    common::Handler,
    file_source::types::{FileIterationInfo, InputFileInfo},
};

use crate::plugin::common::DisposableHnandler;

pub type FileId = String;

pub trait FileSource {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(
        &self,
        param: &serde_json::Value,
        last_indexed_files: &[InputFileInfo],
        access_token: Option<String>,
        download_dir: &str,
    ) -> Result<DisposableHnandler, String>;
    fn next(&self, handler: Handler) -> Result<Option<FileIterationInfo>, String>;
}
