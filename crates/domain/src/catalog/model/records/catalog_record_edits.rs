use indexmap::IndexSet;

use crate::DomainError;
use crate::cases::{Case, CaseId};
use crate::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use crate::contract::{Contract, ContractId};
use crate::document::DocumentId;
use crate::tax::{TaxAssignment, TaxYear};

/// The generic changes to contracts, cases and tax years: the features build the new record and save it here.
impl Catalog {
    /// Adds `contract`, or replaces the contract with the same id in place.
    ///
    /// # Errors
    /// Fails for an invalid contract.
    pub fn save_contract(&self, contract: Contract) -> Result<CatalogUpdate, DomainError> {
        contract.validate()?;
        if contract.referenced_documents().any(|id| !self.documents.contains_key(id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        }
        Ok(self.applied(|catalog| {
            catalog.records.contracts.insert(contract.id.clone(), contract);
        }))
    }

    /// Removes the contract and its items in the emergency folder; its documents stay.
    #[must_use]
    pub fn delete_contract(&self, contract_id: &ContractId) -> CatalogUpdate {
        if !self.records.contracts.contains_key(contract_id) {
            return CatalogUpdate::Rejected(CatalogRejection::ContractNotFound);
        }
        self.applied(|catalog| {
            catalog.records.contracts.shift_remove(contract_id);
            catalog.settings.emergency_folder = catalog
                .settings
                .emergency_folder
                .as_ref()
                .map(|folder| folder.retaining(&|_: &DocumentId| true, &|id: &ContractId| id != contract_id));
        })
    }

    /// Adds `case`, or replaces the case with the same id in place.
    ///
    /// # Errors
    /// Fails for an invalid case.
    pub fn save_case(&self, case: Case) -> Result<CatalogUpdate, DomainError> {
        case.validate()?;
        if case.referenced_documents().any(|id| !self.documents.contains_key(id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        }
        Ok(self.applied(|catalog| {
            catalog.records.cases.insert(case.id.clone(), case);
        }))
    }

    /// Removes the case; its documents stay.
    #[must_use]
    pub fn delete_case(&self, case_id: &CaseId) -> CatalogUpdate {
        if !self.records.cases.contains_key(case_id) {
            return CatalogUpdate::Rejected(CatalogRejection::CaseNotFound);
        }
        self.applied(|catalog| {
            catalog.records.cases.shift_remove(case_id);
        })
    }

    /// Adds `tax_year`, or replaces the checklist of the same year in place.
    ///
    /// # Errors
    /// Fails for an invalid tax year.
    pub fn save_tax_year(&self, tax_year: TaxYear) -> Result<CatalogUpdate, DomainError> {
        tax_year.validate()?;
        if tax_year.referenced_documents().any(|id| !self.documents.contains_key(id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
        }
        Ok(self.applied(|catalog| {
            catalog.records.tax_years.insert(tax_year.year, tax_year);
        }))
    }

    /// Sets or, with `None`, clears a document's place in a tax return.
    ///
    /// # Errors
    /// Fails for an invalid assignment.
    pub fn set_tax(&self, document_id: &DocumentId, tax: Option<TaxAssignment>) -> Result<CatalogUpdate, DomainError> {
        tax.as_ref().map_or(Ok(()), TaxAssignment::validate)?;
        Ok(self.update_document(document_id, IndexSet::new(), |document| document.tax = tax))
    }
}
