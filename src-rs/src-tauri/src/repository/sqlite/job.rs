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
        Ok(()) => {
            active_job.status = Set(STATUS_DONE.to_owned());
            active_job.progress_info = Set("完了".to_owned());
        }
        Err(error) if job.attempts >= job.max_attempts => {
            active_job.status = Set(STATUS_DEAD.to_owned());
            active_job.progress_info = Set(format!("失敗: {error}"));
            active_job.last_error = Set(Some(error));
        }
        Err(error) => {
            // 指数バックオフ（上限付き）
            let delay = (1_000i64 << job.attempts.min(10)).min(600_000);
            active_job.status = Set(STATUS_PENDING.to_owned());
            active_job.progress_info = Set(format!("失敗、再試行待ち: {error}"));
            active_job.run_at = Set(now + Duration::milliseconds(delay));
            active_job.last_error = Set(Some(error));
        }
    }
    active_job.update(db).await?;
    Ok(())
}

pub async fn update_progress(
    db: &DatabaseConnection,
    job_id: i64,
    progress_info: impl Into<String>,
) -> Result<(), String> {
    job::Entity::update_many()
        .col_expr(job::Column::ProgressInfo, Expr::value(progress_info.into()))
        .col_expr(job::Column::UpdatedAt, Expr::value(Utc::now()))
        .filter(job::Column::Id.eq(job_id))
        .exec(db)
        .await
        .map_err(|err| err.to_string())?;
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
