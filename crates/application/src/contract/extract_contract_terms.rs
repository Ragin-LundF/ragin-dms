use std::sync::Arc;

use dms_domain::contract::{ContractId, PricePoint};
use dms_domain::document::Document;

use crate::common::source_texts;
use crate::contract::{ContractTermsSuggestion, ExtractContractTermsOutcome};
use crate::library::CatalogStore;
use crate::port::ai::DocumentContentReader;
use crate::port::assistant::{ContractTermsAnswer, ContractTermsReader, ContractTermsRequest};
use crate::port::state::SettingsRepository;
use crate::port::storage::TextStore;

/// Lets the AI read a contract's prices, notice period and term end from its documents, oldest first (price
/// increase letters are the main source). Only reads; the user accepts each price point and the terms in a review.
pub struct ExtractContractTerms {
    store: Arc<CatalogStore>,
    reader: Arc<dyn DocumentContentReader>,
    texts: Arc<dyn TextStore>,
    settings: Arc<dyn SettingsRepository>,
    terms_reader: Arc<dyn ContractTermsReader>,
}

impl ExtractContractTerms {
    pub fn new(
        store: Arc<CatalogStore>,
        reader: Arc<dyn DocumentContentReader>,
        texts: Arc<dyn TextStore>,
        settings: Arc<dyn SettingsRepository>,
        terms_reader: Arc<dyn ContractTermsReader>,
    ) -> Self {
        Self { store, reader, texts, settings, terms_reader }
    }

    pub fn run(&self, contract_id: &ContractId) -> ExtractContractTermsOutcome {
        let Some(library) = self.store.current() else {
            return ExtractContractTermsOutcome::LibraryNotOpen;
        };
        let Some(contract) = library.catalog.contracts().get(contract_id) else {
            return ExtractContractTermsOutcome::NotFound;
        };
        // Without readable preferences the AI counts as off, as on a new device.
        let ai = self.settings.load_preferences().map(|preferences| preferences.ai).unwrap_or_default();
        if !ai.enabled {
            return ExtractContractTermsOutcome::AiDisabled;
        }
        let documents = library.catalog.documents();
        let mut sources: Vec<&Document> = contract.document_ids.iter().filter_map(|id| documents.get(id)).collect();
        sources
            .sort_by_key(|document| document.metadata.document_date.unwrap_or_else(|| document.added_at.date_naive()));
        let budget = usize::try_from(ai.input_characters).unwrap_or_default();
        let read = source_texts(self.texts.as_ref(), self.reader.as_ref(), library.folders.primary(), &sources, budget);
        if read.is_empty() {
            return ExtractContractTermsOutcome::NoText;
        }
        let request = ContractTermsRequest {
            contract_name: contract.name.clone(),
            correspondent: contract.correspondent.clone(),
            source_texts: read.iter().map(|(_, text)| text.clone()).collect(),
        };
        match self.terms_reader.read(&request) {
            ContractTermsAnswer::Failed { reason, .. } => ExtractContractTermsOutcome::Failed(reason),
            ContractTermsAnswer::Read { reading, model } => {
                let mut prices: Vec<PricePoint> = reading
                    .prices
                    .into_iter()
                    .map(|read_price| PricePoint {
                        valid_from: read_price.valid_from,
                        price: read_price.price,
                        document_id: read_price
                            .source
                            .and_then(|number| read.get(number.checked_sub(1)?))
                            .map(|(document, _)| document.id.clone()),
                    })
                    .filter(|point| {
                        !contract
                            .price_history
                            .iter()
                            .any(|known| known.valid_from == point.valid_from && known.price == point.price)
                    })
                    .collect();
                prices.sort_by_key(|point| point.valid_from);
                prices.dedup_by(|second, first| second.valid_from == first.valid_from);
                ExtractContractTermsOutcome::Suggested(ContractTermsSuggestion {
                    prices,
                    notice: reading.notice,
                    term_end: reading.term_end,
                    renews_by_months: reading.renews_by_months,
                    model,
                })
            }
        }
    }
}
