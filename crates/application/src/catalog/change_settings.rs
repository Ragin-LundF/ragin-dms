use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::settings::CatalogSettings;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Changes the library-wide settings in the catalog, such as the field definitions, saved searches, rules, the
/// default currency or the saved path template, and saves them to every library folder. A change that leaves the
/// settings as they are writes nothing.
pub struct ChangeSettings {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl ChangeSettings {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    /// `change` may run twice, like every edit given to [`CatalogStore::commit`].
    pub fn run(&self, subject: &str, change: impl Fn(&CatalogSettings) -> CatalogSettings) -> OperationOutcome {
        let outcome = self.store.commit(|catalog| {
            let changed = change(catalog.settings());
            if changed == *catalog.settings() {
                Ok(CatalogUpdate::Applied(catalog.clone()))
            } else {
                catalog.change_settings(changed)
            }
        });
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, subject);
        }
        outcome
    }
}
