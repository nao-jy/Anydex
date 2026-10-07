use anydex_plugin_api_lib::{
    common::Handler,
    file_source::{params::GetDiffData, types::InputFileInfo},
};

use crate::plugin::common::DisposableHnandler;

pub type FileId = String;

pub trait FileSource {
    // Return JSON Schema
    fn get_param_schema(&self) -> Result<serde_json::Value, String>;
    fn get_handler(&self, param: &serde_json::Value) -> Result<DisposableHnandler, String>;
    fn get_diff(
        &self,
        handler: Handler,
        last_indexed_files: &[InputFileInfo],
        access_token: Option<String>,
    ) -> Result<GetDiffData, String>;
    fn download(
        &self,
        handler: Handler,
        uri: &str,
        access_token: Option<String>,
        dst_dir: &str,
    ) -> Result<String, String>;
}
