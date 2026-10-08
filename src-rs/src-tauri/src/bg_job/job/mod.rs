use serde_json::Value;

pub mod collect_diff;
pub mod process_file;

#[allow(async_fn_in_trait)]
pub trait Job {
    async fn process(&self, payload: Value) -> Result<(), String>;
}
