use anydex_plugin_api_lib::file_source::interface::Handler;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub type FileId = String;

pub struct InputFileInfo {
    pub file_id: FileId,
    pub size: usize,
    pub updated_at: DateTime<Utc>,
}

pub struct OutputFileInfo {
    pub file_id: FileId,
    pub name: String,
    pub file_path: String,
    pub content_type: String,
    pub size: usize,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

pub enum FileIterationInfo {
    AddedOrDeleted(OutputFileInfo),
    Deleted(FileId),
}

pub trait FileSource {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(
        &self,
        param: &serde_json::Value,
        last_indexed_files: &[InputFileInfo],
        download_dir: &str,
    ) -> Result<Handler, String>;
    fn free_handler(&self, handler: Handler) -> Result<(), String>;

    fn next(&self, handler: Handler) -> Result<Option<FileIterationInfo>, String>;
}
