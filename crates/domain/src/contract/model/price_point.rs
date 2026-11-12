use chrono::NaiveDate;

use crate::contract::ContractPrice;
use crate::document::DocumentId;

/// A price valid from a day on, with the document that announced it, if it is still in the library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PricePoint {
    pub valid_from: NaiveDate,
    pub price: ContractPrice,
    pub document_id: Option<DocumentId>,
}
