use serde_json::Value;

pub mod collect_diff;
pub mod process_file;

pub trait Job {
    fn process(&self, payload: Value) -> Result<(), String>;
}
