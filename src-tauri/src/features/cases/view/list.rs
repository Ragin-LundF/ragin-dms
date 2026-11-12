use chrono::NaiveDate;
use dms_domain::cases::{Case, CaseId, CaseStatus};
use dms_domain::catalog::Catalog;
use dms_domain::iso_date;

use super::detail::present_detail;
use super::status_dto;
use crate::features::cases::dto::{CaseRowDto, CaseStatusPillDto, CasesFilterDto, CasesViewDto};

/// The list of `filter` and the case `selected`, or the list's first when `selected` is not in it. Active cases
/// with the earliest open follow-up come first; done cases by the day they were closed, newest first.
pub fn present_cases(
    catalog: &Catalog,
    filter: CasesFilterDto,
    selected: Option<&CaseId>,
    today: NaiveDate,
) -> CasesViewDto {
    let (mut done, mut active): (Vec<&Case>, Vec<&Case>) =
        catalog.cases().values().partition(|case| case.status == CaseStatus::Done);
    active.sort_by_key(|case| {
        (case.open_follow_ups().next().map_or(NaiveDate::MAX, |follow_up| follow_up.date), &case.title)
    });
    done.sort_by_key(|case| std::cmp::Reverse(case.closed_on));
    let (active_count, done_count) = (count(&active), count(&done));
    let shown = match filter {
        CasesFilterDto::Active => active,
        CasesFilterDto::Done => done,
    };
    let chosen = shown.iter().find(|case| Some(&case.id) == selected).or_else(|| shown.first());
    CasesViewDto {
        active_count,
        done_count,
        detail: chosen.map(|case| present_detail(catalog, case, today)),
        cases: shown.iter().map(|case| row_of(case, today)).collect(),
    }
}

fn count(cases: &[&Case]) -> u32 {
    u32::try_from(cases.len()).unwrap_or(u32::MAX)
}

fn row_of(case: &Case, today: NaiveDate) -> CaseRowDto {
    CaseRowDto {
        id: case.id.to_string(),
        title: case.title.clone(),
        correspondent: case.correspondent.clone(),
        pill: pill_of(case, today),
        next_step: next_step(case, today),
    }
}

/// "Action needed" while a follow-up is due, otherwise the status in a word.
pub(super) fn pill_of(case: &Case, today: NaiveDate) -> CaseStatusPillDto {
    let attention = case.status != CaseStatus::Done && case.open_follow_ups().any(|follow_up| follow_up.date <= today);
    let label = match case.status {
        CaseStatus::Open if attention => "Action needed",
        CaseStatus::Open => "Open",
        CaseStatus::Waiting => "Waiting",
        CaseStatus::Done => "Done",
    };
    CaseStatusPillDto { status: status_dto(case.status), label: label.to_owned(), attention }
}

/// What comes next: the earliest open follow-up, the waiting time, or the day it was closed.
fn next_step(case: &Case, today: NaiveDate) -> String {
    if case.status == CaseStatus::Done {
        return case.closed_on.map_or_else(|| "Done".to_owned(), |day| format!("Done on {}", iso_date(day)));
    }
    if let Some(follow_up) = case.open_follow_ups().next() {
        return format!("{} by {}", follow_up.text, iso_date(follow_up.date));
    }
    match case.waiting_since.filter(|_| case.status == CaseStatus::Waiting) {
        Some(since) => waiting_for(since, today),
        None => "No next step".to_owned(),
    }
}

pub(super) fn waiting_for(since: NaiveDate, today: NaiveDate) -> String {
    match (today - since).num_days() {
        ..=0 => "Waiting for an answer since today".to_owned(),
        1 => "Waiting for an answer for 1 day".to_owned(),
        days => format!("Waiting for an answer for {days} days"),
    }
}
