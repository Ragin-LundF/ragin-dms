use indexmap::IndexSet;

use crate::catalog::Catalog;
use crate::emergency::{EmergencyEntry, EmergencyFolder, EmergencyItemId};

/// What changed in the emergency folder since it was last refreshed: documents whose content differs from the one
/// recorded at the refresh (or that were added after it), and contracts with a new price or an end after it. A folder
/// that was never refreshed has no changes. Removed documents and contracts leave the folder with them, so they never
/// show up here.
pub struct EmergencyChanges;

impl EmergencyChanges {
    #[must_use]
    pub fn of(folder: &EmergencyFolder, catalog: &Catalog) -> IndexSet<EmergencyItemId> {
        let Some(refreshed) = folder.last_refreshed else {
            return IndexSet::new();
        };
        folder
            .items
            .iter()
            .filter(|item| match &item.entry {
                EmergencyEntry::Document(id) => catalog
                    .documents()
                    .get(id)
                    .is_some_and(|document| item.recorded_hash.as_ref() != Some(&document.content_hash)),
                EmergencyEntry::Contract(id) => catalog.contracts().get(id).is_some_and(|contract| {
                    contract.ended_on.is_some_and(|ended| ended > refreshed)
                        || contract.price_history.iter().any(|point| point.valid_from > refreshed)
                }),
                EmergencyEntry::Contact { .. } | EmergencyEntry::Note(_) => false,
            })
            .map(|item| item.id.clone())
            .collect()
    }
}
