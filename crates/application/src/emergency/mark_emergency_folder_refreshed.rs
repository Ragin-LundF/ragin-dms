use std::sync::Arc;

use dms_domain::emergency::EmergencyEntry;
use dms_domain::settings::CatalogSettings;

use crate::catalog::ChangeSettings;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::Clock;

/// Records that the user went through the emergency folder today: the refresh date, and the content of every
/// document item, so later edits of those documents show as changed.
pub struct MarkEmergencyFolderRefreshed {
    store: Arc<CatalogStore>,
    change_settings: Arc<ChangeSettings>,
    clock: Arc<dyn Clock>,
}

impl MarkEmergencyFolderRefreshed {
    pub fn new(store: Arc<CatalogStore>, change_settings: Arc<ChangeSettings>, clock: Arc<dyn Clock>) -> Self {
        Self { store, change_settings, clock }
    }

    pub fn run(&self) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        if library.catalog.settings().emergency_folder.is_none() {
            return OperationOutcome::NotFound;
        }
        let today = self.clock.today();
        let documents = library.catalog.documents();
        self.change_settings.run("Emergency folder refreshed", |settings| {
            let mut folder = settings.emergency_folder.clone().unwrap_or_default();
            folder.last_refreshed = Some(today);
            for item in &mut folder.items {
                if let EmergencyEntry::Document(id) = &item.entry {
                    item.recorded_hash = documents.get(id).map(|document| document.content_hash.clone());
                }
            }
            CatalogSettings { emergency_folder: Some(folder), ..settings.clone() }
        })
    }
}
