use indexmap::IndexMap;

use crate::DomainError;
use crate::cases::{Case, CaseId};
use crate::contract::{Contract, ContractId};
use crate::document::DocumentId;
use crate::error::require;
use crate::tax::TaxYear;

/// Contracts, cases and tax years of a library, each keyed by its id (tax years by the year).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CatalogRecords {
    pub contracts: IndexMap<ContractId, Contract>,
    pub cases: IndexMap<CaseId, Case>,
    pub tax_years: IndexMap<i32, TaxYear>,
}

impl CatalogRecords {
    /// # Errors
    /// Fails when a key does not match its record's id or year, or a record is invalid.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(self.contracts.iter().all(|(id, contract)| id == &contract.id), || {
            "Contract keys must match ids".to_owned()
        })?;
        require(self.cases.iter().all(|(id, case)| id == &case.id), || "Case keys must match ids".to_owned())?;
        require(self.tax_years.iter().all(|(year, tax_year)| *year == tax_year.year), || {
            "Tax year keys must match years".to_owned()
        })?;
        self.contracts.values().try_for_each(Contract::validate)?;
        self.cases.values().try_for_each(Case::validate)?;
        self.tax_years.values().try_for_each(TaxYear::validate)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty() && self.cases.is_empty() && self.tax_years.is_empty()
    }

    /// Every document a record refers to.
    pub fn referenced_documents(&self) -> impl Iterator<Item = &DocumentId> {
        let contracts = self.contracts.values().flat_map(Contract::referenced_documents);
        let cases = self.cases.values().flat_map(Case::referenced_documents);
        let tax_years = self.tax_years.values().flat_map(TaxYear::referenced_documents);
        contracts.chain(cases).chain(tax_years)
    }

    /// The records without any reference to the documents `keep` refuses: contracts and cases drop them, tax items
    /// keep their place without a document.
    #[must_use]
    pub fn retaining_documents(&self, keep: &impl Fn(&DocumentId) -> bool) -> Self {
        Self {
            contracts: self
                .contracts
                .iter()
                .map(|(id, contract)| (id.clone(), contract.retaining_documents(keep)))
                .collect(),
            cases: self.cases.iter().map(|(id, case)| (id.clone(), case.retaining_documents(keep))).collect(),
            tax_years: self
                .tax_years
                .iter()
                .map(|(year, tax_year)| (*year, tax_year.retaining_documents(keep)))
                .collect(),
        }
    }
}
