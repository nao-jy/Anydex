use lancedb::arrow::arrow_schema::DataType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy)]
pub enum VectorType {
    Float16,
    Float32,
}

impl Into<DataType> for VectorType {
    fn into(self) -> DataType {
        match self {
            Self::Float16 => DataType::Float16,
            Self::Float32 => DataType::Float32,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[schemars(title = "SQLite-Vector Params")]
pub struct Param {
    #[schemars(title = "Vector Dimensions", range(min = 1))]
    pub dimensions: i32,
    #[schemars(title = "Vector Type", range(min = 1))]
    pub vector_type: VectorType,
}
