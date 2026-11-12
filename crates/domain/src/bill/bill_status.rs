use chrono::NaiveDate;

use crate::document::Document;
use crate::field::{FieldPresets, FieldRole, FieldValue};
use crate::settings::CatalogSettings;

/// Where a bill stands. A document is a bill when it has a money `amount` and a date field with the role
/// [`FieldRole::DueDate`]; the `paid-on` field marks it as paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BillStatus {
    Open,
    Overdue,
    Paid,
}

impl BillStatus {
    /// The status of `document`, or `None` when it is no bill.
    #[must_use]
    pub fn of(document: &Document, settings: &CatalogSettings, today: NaiveDate) -> Option<Self> {
        let has_amount = matches!(document.fields.get(&FieldPresets::amount().key), Some(FieldValue::Money { .. }));
        let due = Self::due_date(document, settings)?;
        if !has_amount {
            return None;
        }
        if matches!(document.fields.get(&FieldPresets::paid_on().key), Some(FieldValue::Date(_))) {
            return Some(Self::Paid);
        }
        Some(if due < today { Self::Overdue } else { Self::Open })
    }

    /// The earliest date among the document's fields with the role [`FieldRole::DueDate`].
    #[must_use]
    pub fn due_date(document: &Document, settings: &CatalogSettings) -> Option<NaiveDate> {
        document
            .fields
            .iter()
            .filter(|(key, _)| {
                settings.field(key).is_some_and(|definition| definition.role == Some(FieldRole::DueDate))
            })
            .filter_map(|(_, value)| if let FieldValue::Date(date) = value { Some(*date) } else { None })
            .min()
    }
}
