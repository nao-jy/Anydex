use serde::{Deserialize, Serialize};

use crate::{
    common::Handler,
    file_source::interface::{FileIterationInfo, InputFileInfo},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaArgs {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaData {
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHandlerArgs {
    pub params: serde_json::Value,
    pub last_indexed_files: Vec<InputFileInfo>,
    pub access_token: Option<String>,
    pub download_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHandlerData {
    pub handler: Handler,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreeHandlerArgs {
    pub handler: Handler,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreeHandlerData {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextArgs {
    pub handler: Handler,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextData {
    pub info: Option<FileIterationInfo>,
}
