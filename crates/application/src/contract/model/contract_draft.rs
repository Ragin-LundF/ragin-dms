use chrono::NaiveDate;
use dms_domain::contract::{Contract, ContractId, ContractKind, ContractPrice, ContractTerms, PricePoint};
use dms_domain::document::DocumentId;

/// A contract as the user entered it in the editor: `id` is `None` for a new one. A changed `price` with a
/// `price_valid_from` also goes into the price history, and so do `added_prices` (accepted from the AI's reading);
/// `document_ids` are attached in addition to the documents the contract already has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDraft {
    pub id: Option<ContractId>,
    pub name: String,
    pub correspondent: Option<String>,
    pub kind: ContractKind,
    pub price: Option<ContractPrice>,
    pub price_valid_from: Option<NaiveDate>,
    pub terms: ContractTerms,
    pub note: Option<String>,
    pub document_ids: Vec<DocumentId>,
    pub added_prices: Vec<PricePoint>,
}

impl ContractDraft {
    /// The draft that saves `contract` as it is; change a part of it to change only that.
    #[must_use]
    pub fn of(contract: &Contract) -> Self {
        Self {
            id: Some(contract.id.clone()),
            name: contract.name.clone(),
            correspondent: contract.correspondent.clone(),
            kind: contract.kind,
            price: contract.price.clone(),
            price_valid_from: None,
            terms: contract.terms.clone(),
            note: contract.note.clone(),
            document_ids: Vec::new(),
            added_prices: Vec::new(),
        }
    }
}
