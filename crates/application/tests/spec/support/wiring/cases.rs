//! Wiring of the `cases` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::cases::CaseServices;
use dms_application::port::system::{Clock, IdGenerator};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn cases(&self) -> CaseServices {
        let clock: Arc<dyn Clock> = self.clock.clone();
        let ids: Arc<dyn IdGenerator> = self.ids.clone();
        CaseServices::new(&self.store, &clock, &ids)
    }
}
