use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Param {
    #[schemars(description = "dimensions of vectors")]
    pub dimensions: i32,
}
