use std::sync::Arc;

use dms_domain::emergency::{EmergencyEntry, EmergencyFolder, EmergencyItem, EmergencyItemId};
use dms_domain::settings::CatalogSettings;

use crate::catalog::ChangeSettings;
use crate::common::OperationOutcome;
use crate::emergency::EmergencyEdit;
use crate::library::CatalogStore;
use crate::port::system::IdGenerator;

/// Adds, removes, includes or excludes items of the emergency folder, or changes its intro or refresh period. The
/// first added item creates the folder.
pub struct EditEmergencyFolder {
    store: Arc<CatalogStore>,
    change_settings: Arc<ChangeSettings>,
    ids: Arc<dyn IdGenerator>,
}

impl EditEmergencyFolder {
    pub fn new(store: Arc<CatalogStore>, change_settings: Arc<ChangeSettings>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { store, change_settings, ids }
    }

    pub fn run(&self, edit: &EmergencyEdit) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let folder = library.catalog.settings().emergency_folder.clone().unwrap_or_default();
        let target = match edit {
            EmergencyEdit::Remove(id) | EmergencyEdit::SetIncluded { id, .. } => Some(id),
            EmergencyEdit::Add { .. } | EmergencyEdit::SetIntro(_) | EmergencyEdit::SetRefreshMonths(_) => None,
        };
        if target.is_some_and(|id| !folder.items.iter().any(|item| &item.id == id)) {
            return OperationOutcome::NotFound;
        }
        let new_id = EmergencyItemId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
        self.change_settings.run("Emergency folder", |settings| CatalogSettings {
            emergency_folder: Some(edited(settings.emergency_folder.clone().unwrap_or_default(), edit, &new_id)),
            ..settings.clone()
        })
    }
}

fn edited(mut folder: EmergencyFolder, edit: &EmergencyEdit, new_id: &EmergencyItemId) -> EmergencyFolder {
    match edit {
        EmergencyEdit::Add { group, entry } => {
            let present = matches!(entry, EmergencyEntry::Document(_) | EmergencyEntry::Contract(_))
                && folder.items.iter().any(|item| &item.entry == entry);
            if !present {
                folder.items.push(EmergencyItem {
                    id: new_id.clone(),
                    group: *group,
                    entry: entry.clone(),
                    included: true,
                    recorded_hash: None,
                });
            }
        }
        EmergencyEdit::Remove(id) => folder.items.retain(|item| &item.id != id),
        EmergencyEdit::SetIncluded { id, included } => {
            folder.items.iter_mut().filter(|item| &item.id == id).for_each(|item| item.included = *included);
        }
        EmergencyEdit::SetIntro(intro) => folder.intro_text.clone_from(intro),
        EmergencyEdit::SetRefreshMonths(months) => folder.refresh_months = *months,
    }
    folder
}
