use anydex_plugin_api_lib::vector_store::interface::IndexId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Param {
    #[schemars(description = "dimensions of vectors")]
    pub dimensions: i32,
}

pub struct HandleInfo {
    pub param: Param,
    pub index_id: IndexId,
}
