use chrono::NaiveDate;

use crate::cases::{Case, CaseStatus};
use crate::catalog::Catalog;
use crate::task::OpenTask;

/// The dashboard tasks of cases: follow-ups due and cases waiting too long. Closed cases ask for nothing.
pub struct CasePlanner;

impl CasePlanner {
    /// Every open follow-up whose date has come, and every waiting case with no answer for `remind_after_days`.
    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate) -> Vec<OpenTask> {
        catalog
            .cases()
            .values()
            .filter(|case| case.status != CaseStatus::Done)
            .flat_map(|case| tasks_of(case, today))
            .collect()
    }
}

fn tasks_of(case: &Case, today: NaiveDate) -> Vec<OpenTask> {
    let mut tasks: Vec<OpenTask> = case
        .open_follow_ups()
        .filter(|follow_up| follow_up.date <= today)
        .map(|follow_up| OpenTask::FollowUpDue {
            case_id: case.id.clone(),
            follow_up_id: follow_up.id.clone(),
            date: follow_up.date,
        })
        .collect();
    if let Some(since) = waiting_too_long(case, today) {
        tasks.push(OpenTask::CaseWaitingTooLong { case_id: case.id.clone(), since });
    }
    tasks
}

fn waiting_too_long(case: &Case, today: NaiveDate) -> Option<NaiveDate> {
    let since = case.waiting_since.filter(|_| case.status == CaseStatus::Waiting)?;
    ((today - since).num_days() >= i64::from(case.remind_after_days)).then_some(since)
}
