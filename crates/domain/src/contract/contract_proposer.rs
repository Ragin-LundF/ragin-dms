use std::collections::HashSet;

use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::bill::BillStatus;
use crate::catalog::Catalog;
use crate::contract::{ContractProposal, ProposalReason};
use crate::document::{Document, DocumentId};
use crate::expectation::RhythmDetector;
use crate::field::{FieldRole, FieldValue};
use crate::text;

/// Word stems of category names that hold contracts, German and English, compared in lower case.
const CONTRACT_CATEGORY_STEMS: [&str; 8] =
    ["contract", "vertrag", "verträge", "insurance", "versicherung", "subscription", "abonnement", "mitgliedschaft"];
/// Whole words of category names that hold contracts ("Abo" would match "Labor" as a stem).
const CONTRACT_CATEGORY_WORDS: [&str; 2] = ["abo", "abos"];
const MINIMUM_CATEGORY_DOCUMENTS: usize = 2;

/// Proposes to record a correspondent's documents as a contract, never on its own: when a document has a cancel-by
/// or contract-end date, when at least two documents are in a category named like contract, insurance or
/// subscription, or when the correspondent sends bills in a regular rhythm (checked in this order; the first reason
/// wins and its documents go with the proposal). Correspondents that already have a contract (by name, ignoring
/// case, or by one of their documents), and those the user dismissed, are left out.
pub struct ContractProposer;

impl ContractProposer {
    /// The proposals on `today`, by correspondent name.
    #[must_use]
    pub fn proposals(catalog: &Catalog, today: NaiveDate) -> Vec<ContractProposal> {
        let excluded = excluded_correspondents(catalog);
        let attached: HashSet<&DocumentId> = catalog.contracts().values().flat_map(|c| c.document_ids.iter()).collect();
        let rhythmic: HashSet<String> =
            RhythmDetector::expected(catalog, today).iter().map(|expected| key_of(&expected.correspondent)).collect();
        let mut proposals: Vec<ContractProposal> = by_correspondent(catalog)
            .into_iter()
            .filter(|(key, (_, documents))| {
                !excluded.contains(key) && !documents.iter().any(|document| attached.contains(&document.id))
            })
            .filter_map(|(key, (correspondent, documents))| {
                let (reason, matching) = reason_of(catalog, &documents, rhythmic.contains(&key), today)?;
                Some(ContractProposal {
                    correspondent,
                    document_ids: matching.into_iter().map(|document| document.id.clone()).collect(),
                    reason,
                })
            })
            .collect();
        proposals.sort_by(|first, second| text::compare(&key_of(&first.correspondent), &key_of(&second.correspondent)));
        proposals
    }
}

fn key_of(correspondent: &str) -> String {
    text::lowercase(text::trim(correspondent))
}

fn excluded_correspondents(catalog: &Catalog) -> HashSet<String> {
    let recorded = catalog.contracts().values().filter_map(|contract| contract.correspondent.as_deref());
    let dismissed = catalog.settings().dismissed_contract_proposals.iter().map(String::as_str);
    recorded.chain(dismissed).map(key_of).collect()
}

/// The documents per correspondent (lower case), with the spelling of the first document.
fn by_correspondent(catalog: &Catalog) -> IndexMap<String, (String, Vec<&Document>)> {
    let mut groups: IndexMap<String, (String, Vec<&Document>)> = IndexMap::new();
    for document in catalog.documents().values() {
        let Some(correspondent) = document.metadata.correspondent.as_deref().filter(|name| !text::is_blank(name))
        else {
            continue;
        };
        groups
            .entry(key_of(correspondent))
            .or_insert_with(|| (text::trim(correspondent).to_owned(), Vec::new()))
            .1
            .push(document);
    }
    groups
}

fn reason_of<'d>(
    catalog: &Catalog,
    documents: &[&'d Document],
    rhythmic: bool,
    today: NaiveDate,
) -> Option<(ProposalReason, Vec<&'d Document>)> {
    let settings = catalog.settings();
    let with_deadline = matching(documents, |document| {
        document.fields.iter().any(|(key, value)| {
            matches!(value, FieldValue::Date(_))
                && settings
                    .field(key)
                    .and_then(|definition| definition.role)
                    .is_some_and(|role| matches!(role, FieldRole::CancelBy | FieldRole::ContractEnd))
        })
    });
    if !with_deadline.is_empty() {
        return Some((ProposalReason::DeadlineField, with_deadline));
    }
    let in_contract_category = matching(documents, |document| {
        document
            .category_id
            .as_ref()
            .and_then(|id| catalog.categories().get(id))
            .is_some_and(|category| names_contracts(&category.name))
    });
    if in_contract_category.len() >= MINIMUM_CATEGORY_DOCUMENTS {
        return Some((ProposalReason::ContractCategory, in_contract_category));
    }
    let bills = matching(documents, |document| BillStatus::of(document, settings, today).is_some());
    (rhythmic && !bills.is_empty()).then_some((ProposalReason::RecurringBills, bills))
}

fn matching<'d>(documents: &[&'d Document], predicate: impl Fn(&Document) -> bool) -> Vec<&'d Document> {
    documents.iter().copied().filter(|document| predicate(document)).collect()
}

fn names_contracts(category_name: &str) -> bool {
    let name = text::lowercase(category_name);
    CONTRACT_CATEGORY_STEMS.iter().any(|stem| name.contains(stem))
        || name
            .split(|character: char| !character.is_alphanumeric())
            .any(|word| CONTRACT_CATEGORY_WORDS.contains(&word))
}
