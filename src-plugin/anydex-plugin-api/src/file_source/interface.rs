use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    common::{APIResultSerialized, PluginFn},
    file_source::params::*,
};
use std::ffi::c_char;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFileInfo {
    pub file_id: String,
    pub size: usize,
    pub updated_at: Option<DateTime<Utc>>,
    pub file_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputFileInfo {
    pub file_id: String,
    pub name: String,
    pub downloaded_file_path: String,
    pub content_type: String,
    pub size: usize,
    pub updated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub file_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileIterationInfo {
    AddedOrUpdated(OutputFileInfo),
    Deleted(String),
}

/// ABI version implemented by this host.
pub const DYNAMIC_FILE_SOURCE_ABI_VERSION_V1: i32 = 1;

/// ABI table returned by an embedder dynamic library's `get_plugin` entrypoint.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DynamicFileSourceV1 {
    /// Must equal [`DYNAMIC_FILE_SOURCE_ABI_VERSION_V1`].
    pub abi_version: i32,
    /// Size in bytes of the ABI table allocated by the plugin.
    pub struct_size: usize,
    pub get_param_schema:
        unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub get_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,
    pub free_handler: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    pub next: unsafe extern "C" fn(args_serialized: *const c_char) -> APIResultSerialized,

    /// Releases allocations returned by this plugin's ABI functions.
    pub free: unsafe extern "C" fn(value: APIResultSerialized),
}

impl PluginFn<GetParamSchemaArgs, GetParamSchemaData> for DynamicFileSourceV1 {
    fn apply_and_unwrap(&self, args: GetParamSchemaArgs) -> Result<GetParamSchemaData, String> {
        super::super::common::apply_and_unwrap(args, self.get_param_schema, self.free)
    }
}

impl PluginFn<GetHandlerArgs, GetHandlerData> for DynamicFileSourceV1 {
    fn apply_and_unwrap(&self, args: GetHandlerArgs) -> Result<GetHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.get_handler, self.free)
    }
}

impl PluginFn<FreeHandlerArgs, FreeHandlerData> for DynamicFileSourceV1 {
    fn apply_and_unwrap(&self, args: FreeHandlerArgs) -> Result<FreeHandlerData, String> {
        super::super::common::apply_and_unwrap(args, self.free_handler, self.free)
    }
}

impl PluginFn<NextArgs, NextData> for DynamicFileSourceV1 {
    fn apply_and_unwrap(&self, args: NextArgs) -> Result<NextData, String> {
        super::super::common::apply_and_unwrap(args, self.next, self.free)
    }
}

pub enum DynamicFileSource {
    V1(DynamicFileSourceV1),
}
