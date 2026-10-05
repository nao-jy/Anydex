use serde::{Deserialize, Serialize};

use crate::common::Handler;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaArgs {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaData {
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHandlerArgs {
    pub params: serde_json::Value,
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
pub struct EmbedArgs {
    pub handler: Handler,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedData {
    pub embeddings: Vec<f32>,
}
