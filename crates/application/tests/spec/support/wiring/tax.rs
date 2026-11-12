//! Wiring of the `tax` use cases on `TestLibrary`.

use dms_application::tax::{AssignTax, OpenTaxYear, UpdateTaxItem};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn open_tax_year(&self) -> OpenTaxYear {
        OpenTaxYear::new(self.store.clone(), self.clock.clone(), self.ids.clone())
    }

    pub fn update_tax_item(&self) -> UpdateTaxItem {
        UpdateTaxItem::new(self.store.clone(), self.activity.clone(), self.clock.clone(), self.ids.clone())
    }

    pub fn assign_tax(&self) -> AssignTax {
        AssignTax::new(self.store.clone(), self.activity.clone(), self.clock.clone(), self.ids.clone())
    }
}
