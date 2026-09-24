use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[schemars(title = "SQLite-Vector Params")]
pub struct Param {
    #[schemars(title = "Vector Dimensions", description = "Dimensions of vectors")]
    pub dimensions: i32,
}
