use std::{str::FromStr, sync::Arc};

use sea_orm::DatabaseConnection;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::{bg_job::dispatcher, repository::sqlite::job::init_job};

pub async fn init(db: &DatabaseConnection) -> Result<(), String> {
    init_job(db).await.map_err(|err| err.to_string())?;

    let db = db.clone();
    tauri::async_runtime::spawn(async move {
        dispatcher::run(db, 4, Arc::new(Notify::new()), CancellationToken::new()).await;
    });

    Ok(())
}

pub enum JobType {
    CollectDiff,
    ProcessFile,
}

pub struct UnknownJobType;

impl FromStr for JobType {
    type Err = UnknownJobType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CollectDiff" => Ok(Self::CollectDiff),
            "ProcessFile" => Ok(Self::ProcessFile),
            _ => Err(UnknownJobType),
        }
    }
}

impl ToString for JobType {
    fn to_string(&self) -> String {
        match self {
            Self::CollectDiff => "CollectDiff".into(),
            Self::ProcessFile => "ProcessFile".into(),
        }
    }
}
