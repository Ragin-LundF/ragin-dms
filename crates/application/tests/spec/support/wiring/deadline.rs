//! Wiring of the `deadline` use cases on `TestLibrary`.

use dms_application::deadline::SettleRetention;

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn settle_retention(&self) -> SettleRetention {
        SettleRetention::new(self.store.clone(), self.activity.clone(), self.clock.clone())
    }
}
