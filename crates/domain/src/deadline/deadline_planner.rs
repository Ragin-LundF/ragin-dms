use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::catalog::Catalog;
use crate::dates;
use crate::deadline::UpcomingDate;
use crate::field::{FieldRole, FieldValue};
use crate::retention::RetentionDates;
use crate::task::OpenTask;
use crate::text;

/// Turns the deadline fields and the opt-in retention of the documents into dashboard tasks and the "Upcoming"
/// list. A deadline is announced its role's lead time ahead and stays for as long again after it passed; a
/// retention task appears once its date is reached.
pub struct DeadlinePlanner;

impl DeadlinePlanner {
    /// The deadline and retention tasks due on `today`. Pass [`FieldRole::default_lead_days`] as `lead_days` for
    /// the defaults.
    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate, lead_days: impl Fn(FieldRole) -> i32) -> Vec<OpenTask> {
        let deadlines = deadline_dates(catalog).into_iter().filter_map(|(role, upcoming)| {
            let lead = i64::from(lead_days(role));
            let due = upcoming.date <= dates::plus_days(today, lead) && upcoming.date >= dates::minus_days(today, lead);
            let field_key = upcoming.field?;
            due.then_some(OpenTask::DeadlineDue { document_id: upcoming.document_id, field_key, date: upcoming.date })
        });
        let retention = retention_dates(catalog)
            .into_iter()
            .filter(|upcoming| upcoming.date <= today)
            .map(|upcoming| OpenTask::RetentionDue { document_id: upcoming.document_id, date: upcoming.date });
        deadlines.chain(retention).collect()
    }

    /// The next `limit` deadline and retention dates from `today` on, earliest first.
    #[must_use]
    pub fn upcoming(catalog: &Catalog, today: NaiveDate, limit: usize) -> Vec<UpcomingDate> {
        let mut dates: Vec<UpcomingDate> = deadline_dates(catalog)
            .into_iter()
            .map(|(_, upcoming)| upcoming)
            .chain(retention_dates(catalog))
            .filter(|upcoming| upcoming.date >= today)
            .collect();
        dates.sort_by(|first, second| {
            first
                .date
                .cmp(&second.date)
                .then_with(|| text::compare(first.document_id.value(), second.document_id.value()))
        });
        dates.truncate(limit);
        dates
    }
}

fn deadline_dates(catalog: &Catalog) -> Vec<(FieldRole, UpcomingDate)> {
    let roles: IndexMap<_, _> = catalog
        .settings()
        .field_definitions
        .iter()
        .filter_map(|definition| definition.role.map(|role| (&definition.key, role)))
        .collect();
    catalog
        .documents()
        .values()
        .flat_map(|document| {
            document.fields.iter().filter_map(|(key, value)| {
                let role = *roles.get(key)?;
                let FieldValue::Date(date) = value else {
                    return None;
                };
                Some((role, UpcomingDate { document_id: document.id.clone(), field: Some(key.clone()), date: *date }))
            })
        })
        .collect()
}

fn retention_dates(catalog: &Catalog) -> Vec<UpcomingDate> {
    catalog
        .documents()
        .values()
        .filter_map(|document| {
            RetentionDates::due_date(document, catalog).map(|date| UpcomingDate {
                document_id: document.id.clone(),
                field: None,
                date,
            })
        })
        .collect()
}
