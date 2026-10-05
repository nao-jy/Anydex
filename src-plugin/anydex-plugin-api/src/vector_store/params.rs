use serde::{Deserialize, Serialize};

use crate::vector_store::interface::{Chunk, FileId, IndexerId, SearchHit, SearchRequest};
use crate::{common::Handler, vector_store::interface::IndexId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaArgs {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetParamSchemaData {
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHandlerArgs {
    pub params: serde_json::Value,
    pub index_id: IndexId,
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
pub enum DeleteCond {
    ByFileId(FileId),
    ByIndexerId(IndexerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteChunksArgs {
    pub handler: Handler,
    pub condition: DeleteCond,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteChunksData {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertChunksArgs {
    pub handler: Handler,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertChunksData {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearChunksArgs {
    pub handler: Handler,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClearChunksData {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchArgs {
    pub handler: Handler,
    pub search_request: SearchRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchData {
    pub results: Vec<SearchHit>,
}
