use chrono::{Duration, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Set, Statement,
};

use crate::repository::sqlite::models::{
    self,
    job::{self, STATUS_DEAD, STATUS_DONE, STATUS_PENDING, STATUS_RUNNING},
};

pub async fn claim(
    db: &DatabaseConnection,
    limit: usize,
) -> Result<Vec<models::job::Model>, String> {
    let now = Utc::now();
    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        format!(
            r#"
         UPDATE jobs
            SET status = '{STATUS_RUNNING}', attempts = attempts + 1, updated_at = ?1
          WHERE id IN (
                SELECT id FROM jobs
                 WHERE status = '{STATUS_PENDING}' AND run_at <= ?1
                 ORDER BY priority DESC, run_at, id
                 LIMIT ?2)
         RETURNING *"#
        ),
        [now.into(), (limit as i64).into()],
    );
    job::Entity::find()
        .from_raw_sql(stmt)
        .all(db)
        .await
        .map_err(|err| err.to_string())
}

pub async fn finish(
    db: &DatabaseConnection,
    job: &job::Model,
    result: Result<(), String>,
) -> Result<(), sea_orm::DbErr> {
    let now = Utc::now();
    let mut active_job = job::ActiveModel {
        id: Set(job.id),
        updated_at: Set(now),
        ..Default::default()
    };
    match result {
        Ok(()) => active_job.status = Set(STATUS_DONE.to_owned()),
        Err(error) if job.attempts >= job.max_attempts => {
            active_job.status = Set(STATUS_DEAD.to_owned());
            active_job.last_error = Set(Some(error));
        }
        Err(error) => {
            // 指数バックオフ（上限付き）
            let delay = (1_000i64 << job.attempts.min(10)).min(600_000);
            active_job.status = Set(STATUS_PENDING.to_owned());
            active_job.run_at = Set(now + Duration::milliseconds(delay));
            active_job.last_error = Set(Some(error));
        }
    }
    active_job.update(db).await?;
    Ok(())
}

pub async fn init_job(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let cutoff = Utc::now() - Duration::days(30);
    job::Entity::delete_many()
        .filter(job::Column::Status.eq(STATUS_DONE))
        .filter(job::Column::UpdatedAt.lt(cutoff))
        .exec(db)
        .await?;

    job::Entity::update_many()
        .col_expr(job::Column::Status, Expr::value(STATUS_PENDING))
        .filter(job::Column::Status.eq(STATUS_RUNNING))
        .exec(db)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, Set, Statement};

    use crate::repository::sqlite::{initialize, models::job};

    use super::init_job;

    fn new_job(
        status: &str,
        priority: i32,
        run_at: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
    ) -> job::ActiveModel {
        job::ActiveModel {
            index_id: Set("test".to_owned()),
            job_type: Set("test".to_owned()),
            payload: Set(serde_json::json!({})),
            status: Set(status.to_owned()),
            priority: Set(priority),
            attempts: Set(0),
            max_attempts: Set(3),
            run_at: Set(run_at),
            last_error: Set(None),
            created_at: Set(updated_at),
            updated_at: Set(updated_at),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn initialization_prunes_old_done_jobs_and_requeues_running_jobs() {
        let db = initialize("sqlite::memory:").await.unwrap();
        let now = Utc::now();
        for model in [
            new_job(
                super::STATUS_DONE,
                0,
                now - Duration::days(31),
                now - Duration::days(31),
            ),
            new_job(
                super::STATUS_DONE,
                0,
                now - Duration::days(29),
                now - Duration::days(29),
            ),
            new_job(
                super::STATUS_RUNNING,
                0,
                now - Duration::days(31),
                now - Duration::days(31),
            ),
        ] {
            job::Entity::insert(model)
                .exec_without_returning(&db)
                .await
                .unwrap();
        }

        init_job(&db).await.unwrap();

        let jobs = db
            .query_all_raw(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT status FROM jobs ORDER BY status",
            ))
            .await
            .unwrap();
        let statuses: Vec<String> = jobs
            .iter()
            .map(|row| row.try_get("", "status").unwrap())
            .collect();
        assert_eq!(
            statuses,
            [
                super::STATUS_DONE.to_owned(),
                super::STATUS_PENDING.to_owned()
            ]
        );
    }
}
