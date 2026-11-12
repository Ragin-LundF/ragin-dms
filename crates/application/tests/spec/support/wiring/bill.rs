//! Wiring of the `bill` and `expectation` use cases on `TestLibrary`.

use dms_application::bill::{MarkBillPaid, MarkBillUnpaid, SettleOlderBills, SuggestFromEInvoice};
use dms_application::expectation::SnoozeExpectedDocument;

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn mark_bill_paid(&self) -> MarkBillPaid {
        MarkBillPaid::new(self.store.clone(), self.activity.clone())
    }

    pub fn mark_bill_unpaid(&self) -> MarkBillUnpaid {
        MarkBillUnpaid::new(self.store.clone(), self.activity.clone())
    }

    pub fn settle_older_bills(&self) -> SettleOlderBills {
        SettleOlderBills::new(self.store.clone(), self.activity.clone())
    }

    pub fn suggest_from_e_invoice(&self) -> SuggestFromEInvoice {
        SuggestFromEInvoice::new(self.store.clone(), self.reader.clone(), self.tasks.clone())
    }

    pub fn snooze_expected_document(&self) -> SnoozeExpectedDocument {
        SnoozeExpectedDocument::new(self.tasks.clone(), self.clock.clone())
    }
}
