use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use crate::document::DocumentId;
use crate::field::{FieldKey, FieldPresets, FieldType, FieldValue};

/// The catalog changes of bills to pay.
impl Catalog {
    /// Sets (`Some`) or clears (`None`) the "Paid on" date of bills, all in one change. The standard field "Paid on"
    /// is defined on first use.
    #[must_use]
    pub fn set_paid_on(&self, paid_on: &IndexMap<DocumentId, Option<NaiveDate>>) -> CatalogUpdate {
        if paid_on.keys().any(|id| !self.documents.contains_key(id)) {
            return CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound);
        }
        let preset = FieldPresets::paid_on();
        if self.settings.field(&preset.key).is_some_and(|definition| definition.field_type != FieldType::Date) {
            return CatalogUpdate::Rejected(CatalogRejection::FieldTypeMismatch);
        }
        self.applied(|catalog| {
            if catalog.settings.field(&preset.key).is_none() {
                catalog.settings = catalog.settings.with_field(preset.clone());
            }
            for (id, date) in paid_on {
                let Some(document) = catalog.documents.get_mut(id) else { continue };
                match date {
                    Some(date) => document.fields.insert(preset.key.clone(), FieldValue::Date(*date)),
                    None => document.fields.shift_remove(&preset.key),
                };
            }
        })
    }

    /// Defines the standard fields among `keys` the library does not have yet; unchanged when nothing is missing.
    #[must_use]
    pub fn define_standard_fields<'k>(&self, keys: impl IntoIterator<Item = &'k FieldKey>) -> CatalogUpdate {
        let missing: Vec<_> =
            keys.into_iter().filter(|key| self.settings.field(key).is_none()).filter_map(FieldPresets::of).collect();
        if missing.is_empty() {
            return CatalogUpdate::Applied(self.clone());
        }
        self.applied(|catalog| {
            for definition in missing {
                catalog.settings = catalog.settings.with_field(definition);
            }
        })
    }
}
