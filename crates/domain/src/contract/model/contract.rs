use chrono::NaiveDate;
use indexmap::IndexSet;

use crate::DomainError;
use crate::contract::{ContractId, ContractKind, ContractPrice, ContractTerms, PricePoint};
use crate::dates;
use crate::document::DocumentId;
use crate::error::{require_none_or_not_blank, require_not_blank};

/// A contract or subscription the user recorded, with its price, terms and the documents that belong to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub id: ContractId,
    pub name: String,
    pub correspondent: Option<String>,
    pub kind: ContractKind,
    pub price: Option<ContractPrice>,
    pub price_history: Vec<PricePoint>,
    pub terms: ContractTerms,
    pub document_ids: IndexSet<DocumentId>,
    pub ended_on: Option<NaiveDate>,
    pub note: Option<String>,
}

impl Contract {
    /// A running contract without price, terms or documents.
    ///
    /// # Errors
    /// Fails for a blank name.
    pub fn new(id: ContractId, name: impl Into<String>, kind: ContractKind) -> Result<Self, DomainError> {
        let contract = Self {
            id,
            name: name.into(),
            correspondent: None,
            kind,
            price: None,
            price_history: Vec::new(),
            terms: ContractTerms::default(),
            document_ids: IndexSet::new(),
            ended_on: None,
            note: None,
        };
        contract.validate()?;
        Ok(contract)
    }

    /// # Errors
    /// Fails for a blank name, a blank correspondent or note, or invalid terms.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.name, "Contract name")?;
        require_none_or_not_blank(self.correspondent.as_deref(), "Correspondent")?;
        require_none_or_not_blank(self.note.as_deref(), "Note")?;
        self.terms.validate()
    }

    /// `true` while the contract has not ended on `today`; a contract ending later still runs.
    #[must_use]
    pub fn is_running(&self, today: NaiveDate) -> bool {
        self.ended_on.is_none_or(|ended_on| ended_on > today)
    }

    /// The price valid on `date` by the price history; `None` before the first known price point.
    #[must_use]
    pub fn price_on(&self, date: NaiveDate) -> Option<&ContractPrice> {
        self.price_history
            .iter()
            .filter(|point| point.valid_from <= date)
            .max_by_key(|point| point.valid_from)
            .map(|point| &point.price)
    }

    /// The contract with `point` in its price history (replacing a point of the same day), sorted by day. A point
    /// valid on `today` that is the latest so far also becomes the current price.
    #[must_use]
    pub fn with_price_point(&self, point: PricePoint, today: NaiveDate) -> Self {
        let mut contract = self.clone();
        contract.price_history.retain(|known| known.valid_from != point.valid_from);
        contract.price_history.push(point);
        contract.price_history.sort_by_key(|known| known.valid_from);
        if let Some(price) = contract.price_on(today) {
            contract.price = Some(price.clone());
        }
        contract
    }

    /// The price point of the last year (up to `today`) that raised the monthly price over the point before it.
    #[must_use]
    pub fn recent_increase(&self, today: NaiveDate) -> Option<&PricePoint> {
        let year_ago = dates::minus_days(today, 365);
        let mut known = self.price_history.iter().filter(|point| point.valid_from <= today);
        let mut previous = known.next()?;
        let mut increase = None;
        for point in known {
            if point.valid_from > year_ago && point.price.costs_more_than(&previous.price) {
                increase = Some(point);
            }
            previous = point;
        }
        increase
    }

    /// Every document the contract refers to: its documents and the sources of its prices.
    pub fn referenced_documents(&self) -> impl Iterator<Item = &DocumentId> {
        self.document_ids.iter().chain(self.price_history.iter().filter_map(|point| point.document_id.as_ref()))
    }

    /// The contract without any reference to the documents `keep` refuses.
    #[must_use]
    pub fn retaining_documents(&self, keep: &impl Fn(&DocumentId) -> bool) -> Self {
        let mut contract = self.clone();
        contract.document_ids.retain(|id| keep(id));
        for point in &mut contract.price_history {
            point.document_id = point.document_id.take().filter(|id| keep(id));
        }
        contract
    }
}
