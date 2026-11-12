use std::ops::Deref;
use std::sync::Arc;

use chrono::NaiveDate;
use dms_application::contract::{
    AttachDocumentsToContract, ContractDraft, DeleteContract, DismissContractProposal, EndContract,
    ExtractContractTerms, SaveContract,
};
use dms_domain::contract::{ContractId, ContractKind, ContractPrice, ContractTerms};
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::field::{CurrencyCode, Decimal};
use dms_domain::money::BillingPeriod;

use crate::support::TestLibrary;
use crate::support::fake::assistant::FakeContractTermsReader;
use crate::support::test_data::done;

/// A `TestLibrary` with the contract terms reader fake.
pub struct ContractKit {
    pub library: TestLibrary,
    pub terms_reader: Arc<FakeContractTermsReader>,
}

impl Deref for ContractKit {
    type Target = TestLibrary;

    fn deref(&self) -> &TestLibrary {
        &self.library
    }
}

impl ContractKit {
    pub fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        Self { library, terms_reader: Arc::new(FakeContractTermsReader::default()) }
    }

    pub fn save_contract(&self) -> SaveContract {
        SaveContract::new(self.store.clone(), self.activity.clone(), self.clock.clone(), self.ids.clone())
    }

    pub fn end_contract(&self) -> EndContract {
        EndContract::new(self.store.clone(), self.activity.clone())
    }

    pub fn delete_contract(&self) -> DeleteContract {
        DeleteContract::new(self.store.clone(), self.activity.clone())
    }

    pub fn attach_documents(&self) -> AttachDocumentsToContract {
        AttachDocumentsToContract::new(self.store.clone(), self.activity.clone())
    }

    pub fn dismiss_proposal(&self) -> DismissContractProposal {
        DismissContractProposal::new(Arc::new(self.change_settings()))
    }

    pub fn extract_terms(&self) -> ExtractContractTerms {
        ExtractContractTerms::new(
            self.store.clone(),
            self.reader.clone(),
            self.folders.clone(),
            self.settings.clone(),
            self.terms_reader.clone(),
        )
    }

    pub fn enable_ai(&self) {
        self.settings.preferences.update(|preferences| preferences.ai.enabled = true);
    }

    /// Records a contract from `draft` and returns its id.
    pub fn record(&self, draft: &ContractDraft) -> ContractId {
        let saved = self.save_contract().run(draft);
        done(saved.outcome);
        saved.contract_id
    }

    /// Adds a document dated `date` whose text layer is `text`.
    pub fn add_dated(&self, raw_path: &str, date: NaiveDate, text: &str) -> Document {
        let document = self.add(raw_path, raw_path);
        self.reader.set_text(document.path.file_name(), text);
        let metadata = DocumentMetadata { title: None, document_date: Some(date), correspondent: None };
        done(self.store.commit(|catalog| catalog.update_metadata(&document.id, metadata.clone())));
        self.catalog().documents().get(&document.id).cloned().expect("still catalogued")
    }
}

pub fn euros(amount: &str, period: BillingPeriod) -> ContractPrice {
    ContractPrice {
        amount: Decimal::parse(amount).expect("decimal"),
        currency: CurrencyCode::new("EUR").expect("currency"),
        period,
    }
}

pub fn draft(name: &str) -> ContractDraft {
    ContractDraft {
        id: None,
        name: name.to_owned(),
        correspondent: Some("Net Provider".to_owned()),
        kind: ContractKind::Telecom,
        price: None,
        price_valid_from: None,
        terms: ContractTerms::default(),
        note: None,
        document_ids: Vec::new(),
        added_prices: Vec::new(),
    }
}
