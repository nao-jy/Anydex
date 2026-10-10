use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod collect_diff;
pub mod process_file;

#[allow(async_fn_in_trait)]
pub trait Job {
    type JobArgsType: Serialize + for<'a> Deserialize<'a>;
    async fn process(&self, payload: Value) -> Result<(), String>;
}
