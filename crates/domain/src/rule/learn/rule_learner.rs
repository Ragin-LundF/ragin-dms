use std::collections::HashSet;
use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

use crate::catalog::Catalog;
use crate::document::Document;
use crate::rule::{RuleCondition, RuleProposal};
use crate::text;

/// Proposes rules from the documents the user approved: a correspondent with at least
/// [`RuleLearner::MIN_DOCUMENTS`] classified documents of which at least 90 % share a category, or tags that at
/// least 80 % share. Correspondents that already have a rule are left out; rejected proposals are the caller's
/// to drop.
pub struct RuleLearner;

const CATEGORY_SHARE: f64 = 0.9;
const TAG_SHARE: f64 = 0.8;

impl RuleLearner {
    pub const MIN_DOCUMENTS: usize = 3;

    #[must_use]
    pub fn proposals(catalog: &Catalog) -> Vec<RuleProposal> {
        let covered: HashSet<String> = catalog
            .settings()
            .rules
            .iter()
            .flat_map(|rule| &rule.conditions)
            .filter_map(|condition| match condition {
                RuleCondition::CorrespondentIs { name } => Some(key_of(name)),
                _ => None,
            })
            .collect();
        let mut by_correspondent: IndexMap<String, Vec<&Document>> = IndexMap::new();
        for document in catalog.documents().values() {
            if let (Some(_), Some(correspondent)) = (&document.classification, &document.metadata.correspondent) {
                by_correspondent.entry(key_of(correspondent)).or_default().push(document);
            }
        }
        let mut proposals: Vec<RuleProposal> = by_correspondent
            .into_iter()
            .filter(|(key, documents)| !covered.contains(key) && documents.len() >= Self::MIN_DOCUMENTS)
            .filter_map(|(_, documents)| proposal_of(&documents))
            .collect();
        proposals.sort_by(|first, second| {
            text::compare(&text::lowercase(&first.correspondent), &text::lowercase(&second.correspondent))
        });
        proposals
    }
}

#[allow(clippy::cast_precision_loss)] // document counts are far below 2^52
fn proposal_of(documents: &[&Document]) -> Option<RuleProposal> {
    let total = documents.len() as f64;
    let category_id = first_most_common(counts(documents.iter().filter_map(|document| document.category_id.as_ref())))
        .filter(|(_, count)| *count as f64 >= total * CATEGORY_SHARE)
        .map(|(id, _)| id.clone());
    let tags: IndexSet<_> = counts(documents.iter().flat_map(|document| &document.tags))
        .into_iter()
        .filter(|(_, count)| *count as f64 >= total * TAG_SHARE)
        .map(|(tag, _)| tag.clone())
        .collect();
    if category_id.is_none() && tags.is_empty() {
        return None;
    }
    // The spelling most documents use.
    let name =
        first_most_common(counts(documents.iter().filter_map(|document| document.metadata.correspondent.as_ref())))?;
    Some(RuleProposal { correspondent: name.0.clone(), category_id, tags, document_count: documents.len() })
}

/// How often each item occurs, in the order of first occurrence (Kotlin's `groupingBy { it }.eachCount()`).
fn counts<T: Eq + Hash>(items: impl Iterator<Item = T>) -> IndexMap<T, usize> {
    let mut counts = IndexMap::new();
    for item in items {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
}

/// The first entry with the highest count, like Kotlin's `maxByOrNull`.
fn first_most_common<T>(counts: IndexMap<T, usize>) -> Option<(T, usize)> {
    counts.into_iter().fold(None, |best, (item, count)| match best {
        Some((_, best_count)) if best_count >= count => best,
        _ => Some((item, count)),
    })
}

fn key_of(name: &str) -> String {
    text::lowercase(text::trim(name))
}
