use anydex_plugin_api_lib::common::Handler;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::plugin::common::DisposableHnandler;

pub type FileId = String;

pub struct InputFileInfo {
    pub file_id: FileId,
    pub size: usize,
    pub updated_at: DateTime<Utc>,
    pub file_hash: u64,
}

pub struct OutputFileInfo {
    pub file_id: FileId,
    pub name: String,
    pub file_path: String,
    pub content_type: String,
    pub size: usize,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub file_hash: u64,
}

pub enum FileIterationInfo {
    AddedOrUpdated(OutputFileInfo),
    Deleted(FileId),
}

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
