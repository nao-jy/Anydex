use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

pub mod collect_diff;
pub mod process_file;

#[derive(Debug)]
pub enum JobError {
    Retryable(String),
    NonRetryable(String),
}

impl JobError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self::Retryable(message.into())
    }

    pub fn non_retryable(message: impl Into<String>) -> Self {
        Self::NonRetryable(message.into())
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Retryable(_))
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retryable(message) | Self::NonRetryable(message) => formatter.write_str(message),
        }
    }
}

impl From<String> for JobError {
    fn from(message: String) -> Self {
        Self::retryable(message)
    }
}

impl From<&str> for JobError {
    fn from(message: &str) -> Self {
        Self::retryable(message)
    }
}

#[allow(async_fn_in_trait)]
pub trait Job {
    type JobArgsType: Serialize + for<'a> Deserialize<'a>;
    async fn process(&self, job_id: i64, payload: Value) -> Result<(), JobError>;
}
