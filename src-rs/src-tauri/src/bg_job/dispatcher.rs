use std::{sync::Arc, time::Duration};

use crate::repository::sqlite::{
    job::{claim, finish},
    models::job,
};
use sea_orm::DatabaseConnection;
use tokio::{
    sync::{Notify, Semaphore},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

pub async fn run(
    db: DatabaseConnection,
    worker_n: usize,
    notify: Arc<Notify>,
    cancel: CancellationToken,
) {
    let sem = Arc::new(Semaphore::new(worker_n));
    let mut tasks = JoinSet::new();
    while !cancel.is_cancelled() {
        let free = sem.available_permits();
        let jobs = if free > 0 {
            claim(&db, free, 60_000).await.unwrap_or_default()
        } else {
            vec![]
        };

        if jobs.is_empty() {
            tokio::select! {
                _ = notify.notified() => {},
                _ = tokio::time::sleep(Duration::from_millis(500)) => {},
                Some(_) = tasks.join_next(), if !tasks.is_empty() => {},
                _ = cancel.cancelled() => break,
            }
            continue;
        }

        for job in jobs {
            let permit = sem.clone().acquire_owned().await.unwrap();
            let db = db.clone();
            tasks.spawn(async move {
                let _permit = permit;
                let result = execute(db.clone(), job.clone()).await;
                finish(&db, &job, result).await;
            });
        }
        while tasks.try_join_next().is_some() {}
    }
    while tasks.join_next().await.is_some() {}
}

async fn execute(db: DatabaseConnection, job: job::Model) -> Result<(), String> {
    // tokio::task::spawn_blocking(move || ).await.map_err(|err|err.to_string())
    Ok(())
}
