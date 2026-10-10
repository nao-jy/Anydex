use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::Duration,
};

use chrono::{DateTime, Utc};
use rrule::RRuleSet;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use tokio_util::sync::CancellationToken;

use crate::{
    repository::{
        json,
        sqlite::models::{
            indexing_history,
            job::{self, STATUS_RUNNING},
        },
    },
    usecase::jobs::start_indexing,
};

const CHECK_INTERVAL: Duration = Duration::from_secs(30);

pub async fn run(db: DatabaseConnection, cancel: CancellationToken) {
    let mut last_check = None;

    loop {
        if cancel.is_cancelled() {
            return;
        }

        let now = Utc::now();
        if let Err(error) = check_schedules(&db, last_check, now).await {
            tracing::error!(%error, "failed to check indexer schedules");
        }
        last_check = Some(now);

        tokio::select! {
            _ = tokio::time::sleep(CHECK_INTERVAL) => {},
            _ = cancel.cancelled() => return,
        }
    }
}

async fn check_schedules(
    db: &DatabaseConnection,
    last_check: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<(), String> {
    let userdata = json::load()?;
    let mut histories = HashMap::new();
    let history_rows = indexing_history::Entity::find()
        .order_by_asc(indexing_history::Column::StartedAt)
        .all(db)
        .await
        .map_err(|err| err.to_string())?;
    for history in history_rows {
        histories.insert(history.index_id, history.started_at);
    }

    let running_jobs = job::Entity::find()
        .filter(job::Column::Status.eq(STATUS_RUNNING))
        .all(db)
        .await
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(|job| job.index_id)
        .collect::<HashSet<_>>();

    for index in userdata.indexes {
        let Some(schedule) = index.schedule else {
            continue;
        };
        let last_indexed_at = histories.get(&index.id.to_string()).copied();
        let due = match is_due(
            &schedule.rfc5545_rrule,
            schedule.catchup_when_missed,
            last_indexed_at,
            last_check,
            now,
        ) {
            Ok(due) => due,
            Err(error) => {
                tracing::warn!(index_id = %index.id, %error, "invalid indexer schedule");
                continue;
            }
        };
        if should_start_indexing(due, running_jobs.contains(&index.id.to_string())) {
            if let Err(error) = start_indexing(db, index.id).await {
                tracing::error!(index_id = %index.id, %error, "failed to start scheduled indexing");
            }
        }
    }

    Ok(())
}

fn should_start_indexing(schedule_is_due: bool, has_running_job: bool) -> bool {
    schedule_is_due && !has_running_job
}

fn is_due(
    schedule: &str,
    catchup_when_missed: bool,
    last_indexed_at: Option<DateTime<Utc>>,
    last_check: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<bool, String> {
    let rule = RRuleSet::from_str(schedule).map_err(|err| err.to_string())?;
    let timezone = rule.get_dt_start().timezone();
    let now_in_schedule_timezone = now.with_timezone(&timezone);

    let after = if catchup_when_missed {
        [last_indexed_at, last_check]
            .into_iter()
            .flatten()
            .max()
            .map(|date| date.with_timezone(&timezone))
    } else {
        let lower_bound = [last_indexed_at, last_check]
            .into_iter()
            .flatten()
            .max()
            .unwrap_or(now);
        Some(lower_bound.with_timezone(&timezone))
    };

    let mut rule = rule.before(now_in_schedule_timezone);
    if let Some(after) = after {
        rule = rule.after(after);
    }
    Ok(!rule.all(1).dates.is_empty())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::{is_due, should_start_indexing};

    fn schedule(freq: &str, start: &str) -> String {
        format!("DTSTART:{start}\nRRULE:FREQ={freq}")
    }

    #[test]
    fn catchup_runs_when_occurrence_was_missed_since_last_indexing() {
        let now = Utc.with_ymd_and_hms(2026, 10, 11, 12, 0, 0).unwrap();
        let last_indexed_at = now - Duration::days(2);
        let rule = schedule("DAILY", "20261001T130000Z");

        assert!(is_due(&rule, true, Some(last_indexed_at), None, now).unwrap());
    }

    #[test]
    fn catchup_does_not_run_when_no_occurrence_was_missed() {
        let now = Utc.with_ymd_and_hms(2026, 10, 11, 12, 0, 0).unwrap();
        let last_indexed_at = now - Duration::minutes(1);
        let rule = schedule("DAILY", "20261001T115800Z");

        assert!(!is_due(&rule, true, Some(last_indexed_at), None, now).unwrap());
    }

    #[test]
    fn non_catchup_skips_occurrences_before_the_monitor_started() {
        let now = Utc.with_ymd_and_hms(2026, 10, 11, 12, 0, 0).unwrap();
        let last_check = now - Duration::minutes(1);
        let rule = schedule("DAILY", "20261001T115800Z");

        assert!(!is_due(&rule, false, None, Some(last_check), now).unwrap());
    }

    #[test]
    fn non_catchup_runs_occurrences_since_last_check() {
        let now = Utc.with_ymd_and_hms(2026, 10, 11, 12, 0, 0).unwrap();
        let last_check = now - Duration::minutes(1);
        let rule = schedule("MINUTELY", "20261011T115900Z");

        assert!(is_due(&rule, false, None, Some(last_check), now).unwrap());
    }

    #[test]
    fn rejects_invalid_schedule_rules() {
        let now = Utc::now();

        assert!(is_due("not an rrule", true, None, None, now).is_err());
    }

    #[test]
    fn skips_a_due_schedule_while_the_index_has_a_running_job() {
        assert!(!should_start_indexing(true, true));
    }

    #[test]
    fn starts_a_due_schedule_when_the_index_has_no_running_job() {
        assert!(should_start_indexing(true, false));
    }

    #[test]
    fn does_not_catch_up_a_schedule_skipped_while_a_job_was_running() {
        let now = Utc.with_ymd_and_hms(2026, 10, 11, 12, 1, 0).unwrap();
        let last_indexed_at = now - Duration::days(1);
        let last_check = now - Duration::seconds(30);
        let rule = schedule("DAILY", "20261001T120000Z");

        assert!(!is_due(&rule, true, Some(last_indexed_at), Some(last_check), now).unwrap());
    }
}
