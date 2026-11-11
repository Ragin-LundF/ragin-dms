//! Wiring of the `rule` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::rule::AcceptRuleProposal;

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn accept_rule_proposal(&self) -> AcceptRuleProposal {
        AcceptRuleProposal::new(Arc::new(self.change_settings()), self.ids.clone())
    }
}
