use std::sync::Arc;

use dms_domain::settings::CatalogSettings;
use dms_domain::text;

use crate::catalog::ChangeSettings;
use crate::common::OperationOutcome;

/// Stops proposing a contract for a correspondent, on every device: the name goes into the catalog settings
/// (compared ignoring case, so a second dismissal changes nothing).
pub struct DismissContractProposal {
    change_settings: Arc<ChangeSettings>,
}

impl DismissContractProposal {
    pub fn new(change_settings: Arc<ChangeSettings>) -> Self {
        Self { change_settings }
    }

    pub fn run(&self, correspondent: &str) -> OperationOutcome {
        let name = text::trim(correspondent).to_owned();
        if name.is_empty() {
            return OperationOutcome::NotFound;
        }
        let key = text::lowercase(&name);
        self.change_settings.run(&name, |settings| {
            if settings.dismissed_contract_proposals.iter().any(|known| text::lowercase(text::trim(known)) == key) {
                return settings.clone();
            }
            let mut dismissed = settings.dismissed_contract_proposals.clone();
            dismissed.push(name.clone());
            CatalogSettings { dismissed_contract_proposals: dismissed, ..settings.clone() }
        })
    }
}
