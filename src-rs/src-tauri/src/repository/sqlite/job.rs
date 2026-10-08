use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, Statement};

use crate::repository::sqlite::models::{
    self,
    job::{self, STATUS_DEAD, STATUS_DONE, STATUS_PENDING, STATUS_RUNNING},
};

pub async fn claim(
    db: &DatabaseConnection,
    limit: usize,
    lease_ms: i64,
) -> Result<Vec<models::job::Model>, String> {
    let now = Utc::now().timestamp_millis();
    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        format!(
            r#"
         UPDATE jobs
            SET status = '{STATUS_RUNNING}', attempts = attempts + 1,
                locked_until = ?1 + ?2, updated_at = ?1
          WHERE id IN (
                SELECT id FROM jobs
                 WHERE status = '{STATUS_PENDING}' AND run_at <= ?1
                 ORDER BY priority DESC, run_at, id
                 LIMIT ?4)
         RETURNING *"#
        ),
        [now.into(), lease_ms.into(), (limit as i64).into()],
    );
    job::Entity::find()
        .from_raw_sql(stmt)
        .all(db)
        .await
        .map_err(|err| err.to_string())
}

pub async fn finish(db: &DatabaseConnection, job: &job::Model, r: Result<(), String>) {
    let now = Utc::now().timestamp_millis();
    let (sql, vals): (String, Vec<sea_orm::Value>) = match r {
         Ok(()) => (format!("UPDATE jobs SET status='{STATUS_DONE}', locked_until=NULL, updated_at=?1 WHERE id=?2"),
                    vec![now.into(), job.id.into()]),
         Err(e) if job.attempts >= job.max_attempts =>
             (format!("UPDATE jobs SET status='{STATUS_DEAD}', last_error=?3, locked_until=NULL, updated_at=?1 WHERE id=?2"),
              vec![now.into(), job.id.into(), e.into()]),
         Err(e) => {
             // 指数バックオフ（上限付き）
             let delay = (1_000i64 << job.attempts.min(10)).min(600_000);
             (format!("UPDATE jobs SET status='{STATUS_PENDING}', run_at=?1+?4, last_error=?3, locked_until=NULL, updated_at=?1 WHERE id=?2"),
              vec![now.into(), job.id.into(), e.into(), delay.into()])
         }
     };
    let stmt = Statement::from_sql_and_values(DbBackend::Sqlite, sql, vals);
    let _ = db.execute_raw(stmt).await;
}

pub async fn collect_leased(db: &DatabaseConnection) {
    let stmt = Statement::from_string(
        DbBackend::Sqlite,
        r#"
            UPDATE jobs SET status='pending', locked_until=NULL
            WHERE status='running' AND locked_until < :now
        "#,
    );
    let _ = db.execute_raw(stmt).await;
}

pub async fn heartbeat(db: &DatabaseConnection, job_id: u64, lease_ms: i64) {
    let now = Utc::now().timestamp_millis();
    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        r#"
            UPDATE jobs SET locked_until=?1
            WHERE status='running' AND id=?2
        "#,
        vec![(now + lease_ms).into(), job_id.into()],
    );
    let _ = db.execute_raw(stmt).await;
}
