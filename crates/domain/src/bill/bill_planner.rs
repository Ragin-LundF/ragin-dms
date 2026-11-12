use chrono::NaiveDate;

use crate::bill::BillStatus;
use crate::catalog::Catalog;
use crate::dates;
use crate::document::{Document, DocumentId};
use crate::field::FieldRole;
use crate::settings::CatalogSettings;
use crate::task::OpenTask;

/// The dashboard tasks of bills to pay: one [`OpenTask::BillOverdue`] per unpaid bill whose due date passed. It
/// replaces the deadline task of that due date.
///
/// Bills due before [`CatalogSettings::bills_tracked_since`] are "older bills never marked" and get no task. Before
/// the user opened "To pay" for the first time, only bills that fell due within the due date's lead time count, as
/// the deadline task did, so an old library does not flood the dashboard.
pub struct BillPlanner;

impl BillPlanner {
    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate) -> Vec<OpenTask> {
        let settings = catalog.settings();
        let tracked_from = Self::tracked_from(settings, today);
        catalog
            .documents()
            .values()
            .filter_map(|document| {
                let due = overdue_since(document, settings, today)?;
                (due >= tracked_from).then(|| OpenTask::BillOverdue { document_id: document.id.clone(), due })
            })
            .collect()
    }

    /// The unpaid bills due before the user started tracking bills, with their due date, by due date: the "older
    /// bills never marked". None before tracking started.
    #[must_use]
    pub fn never_marked(catalog: &Catalog) -> Vec<(DocumentId, NaiveDate)> {
        let settings = catalog.settings();
        let Some(since) = settings.bills_tracked_since else { return Vec::new() };
        let mut older: Vec<(DocumentId, NaiveDate)> = catalog
            .documents()
            .values()
            .filter(|document| BillStatus::of(document, settings, since) == Some(BillStatus::Overdue))
            .filter_map(|document| Some((document.id.clone(), BillStatus::due_date(document, settings)?)))
            .collect();
        older.sort_by_key(|(_, due)| *due);
        older
    }

    /// The first due date that counts for tasks: the day tracking started, or the due date's lead time ago.
    #[must_use]
    pub fn tracked_from(settings: &CatalogSettings, today: NaiveDate) -> NaiveDate {
        settings
            .bills_tracked_since
            .unwrap_or_else(|| dates::minus_days(today, i64::from(FieldRole::DueDate.default_lead_days())))
    }
}

fn overdue_since(document: &Document, settings: &CatalogSettings, today: NaiveDate) -> Option<NaiveDate> {
    match BillStatus::of(document, settings, today)? {
        BillStatus::Overdue => BillStatus::due_date(document, settings),
        BillStatus::Open | BillStatus::Paid => None,
    }
}
