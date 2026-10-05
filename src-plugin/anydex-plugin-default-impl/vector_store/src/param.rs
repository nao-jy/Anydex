use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "SQLite-Vector Params")]
pub struct Param {
    #[schemars(title = "Vector Dimensions", range(min = 1))]
    pub dimensions: i32,
}
