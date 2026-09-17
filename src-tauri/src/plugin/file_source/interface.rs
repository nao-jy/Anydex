use axum::Error;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};

use libloading::Library;

use crate::plugin::manager::LibraryLoader;

type Handler = u64;
type FileId = i32;

pub struct FileInfo {
    pub uri: String, // id
    pub name: String,
    pub content_type: String,
    pub size: u64,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

pub struct FileInfoPath {
    pub info: FileInfo,
    pub file_path: String,
}

pub trait FileSource {
    // Return JSON Schema
    fn get_param_schema(&self) -> serde_json::Value;
    fn get_handler(
        &self,
        param: &serde_json::Value,
        last_indexed_files: [FileInfo],
        download_dir: &str,
    ) -> Handler;
    fn free_handler(&self, handler: Handler);

    fn next_added_or_changed(&self, handler: Handler) -> Result<Option<FileInfoPath>, Error>;
    fn next_deleted(&self, handler: Handler) -> Result<Option<FileId>, Error>;
}
