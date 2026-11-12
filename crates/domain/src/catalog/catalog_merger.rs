use std::collections::HashSet;
use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

use crate::DomainError;
use crate::catalog::{Catalog, CatalogMergeConflict, CatalogMergeResult, CatalogRecords};
use crate::document::{Document, DocumentId};
use crate::error::require;
use crate::library::RelativePath;
use crate::link::DocumentLink;
use crate::rule::Correspondent;
use crate::settings::CatalogSettings;
use crate::taxonomy::{Category, CategoryId};
use crate::text;

/// Merges two copies of the same catalog, for example the copies of two library folders, or a conflict copy a
/// sync client created when two devices saved at the same time.
///
/// There is no common ancestor to compare against, so an entry missing on one side cannot be told apart from an
/// entry deleted there. The merge therefore never deletes: it is the union of both copies. Where both copies
/// hold different versions of the same entry, the copy with the higher revision wins (the first one on a tie)
/// and the decision is reported as a conflict.
///
/// Links are merged as a union of pairs. Field definitions, saved searches and rules are merged per id like
/// categories; correspondents are a union by name; the newer copy's layout wins. Contracts, cases and tax years are
/// merged per id (tax years per year) and the owner and the emergency folder as a whole, the newer copy's version
/// winning with a reported conflict, or the older's where the newer has none. Bills are tracked since the earlier
/// of both days; dismissed contract proposals are a union, expectations a union by sender and category with the
/// newer copy winning.
pub struct CatalogMerger;

impl CatalogMerger {
    /// # Errors
    /// Fails for copies of different libraries.
    pub fn merge(first: &Catalog, second: &Catalog) -> Result<CatalogMergeResult, DomainError> {
        require(first.library_id() == second.library_id(), || {
            "Only copies of the same library can be merged".to_owned()
        })?;
        let (newer, older) = if second.revision() > first.revision() { (second, first) } else { (first, second) };
        let mut conflicts = Vec::new();
        let categories = merge_categories(newer.categories(), older.categories(), &mut conflicts);
        let documents = merge_documents(newer.documents(), older.documents(), &mut conflicts);
        let settings = merge_settings(newer.settings(), older.settings(), &mut conflicts);
        let tags = newer.tags().iter().chain(older.tags()).cloned().collect();
        let links = merge_links(newer.links(), older.links(), &documents);
        let records = merge_records(newer.records(), older.records(), &mut conflicts);
        let merged = newer.with_parts(documents, categories, tags, links, settings, records);
        if &merged == newer {
            return Ok(CatalogMergeResult { catalog: newer.clone(), conflicts });
        }
        let revision = newer.revision().next();
        Ok(CatalogMergeResult { catalog: merged.with_revision(revision), conflicts })
    }
}

// Categories only in the older copy cannot form a cycle with the newer copy's tree: nothing in the newer copy
// can point at an id it does not know.
fn merge_categories(
    newer: &IndexMap<CategoryId, Category>,
    older: &IndexMap<CategoryId, Category>,
    conflicts: &mut Vec<CatalogMergeConflict>,
) -> IndexMap<CategoryId, Category> {
    let mut entries = newer.clone();
    for category in older.values() {
        match entries.get(&category.id) {
            None => {
                entries.insert(category.id.clone(), category.clone());
            }
            Some(kept) if kept != category => conflicts
                .push(CatalogMergeConflict::CategoryDiffers { kept: kept.clone(), discarded: category.clone() }),
            Some(_) => {}
        }
    }
    entries
}

fn merge_documents(
    newer: &IndexMap<DocumentId, Document>,
    older: &IndexMap<DocumentId, Document>,
    conflicts: &mut Vec<CatalogMergeConflict>,
) -> IndexMap<DocumentId, Document> {
    let mut entries = newer.clone();
    let mut owner_by_path: IndexMap<RelativePath, Document> =
        newer.values().map(|document| (document.path.clone(), document.clone())).collect();
    for document in older.values() {
        match (entries.get(&document.id), owner_by_path.get(&document.path)) {
            (Some(same_id), _) if same_id != document => conflicts
                .push(CatalogMergeConflict::DocumentDiffers { kept: same_id.clone(), discarded: document.clone() }),
            (Some(_), _) => {}
            (None, Some(same_path)) => {
                conflicts
                    .push(CatalogMergeConflict::PathClash { kept: same_path.clone(), discarded: document.clone() });
            }
            (None, None) => {
                entries.insert(document.id.clone(), document.clone());
                owner_by_path.insert(document.path.clone(), document.clone());
            }
        }
    }
    entries
}

/// The union of both copies' links between documents the merged catalog has; the newer label wins.
fn merge_links(
    newer: &IndexSet<DocumentLink>,
    older: &IndexSet<DocumentLink>,
    documents: &IndexMap<DocumentId, Document>,
) -> IndexSet<DocumentLink> {
    let older_only = older.iter().filter(|link| !newer.iter().any(|it| it.same_pair(link)));
    newer
        .iter()
        .chain(older_only)
        .filter(|link| documents.contains_key(link.first()) && documents.contains_key(link.second()))
        .cloned()
        .collect()
}

fn merge_settings(
    newer: &CatalogSettings,
    older: &CatalogSettings,
    conflicts: &mut Vec<CatalogMergeConflict>,
) -> CatalogSettings {
    let field_definitions = union_by_id(
        &newer.field_definitions,
        &older.field_definitions,
        |it| it.key.clone(),
        conflicts,
        |kept, discarded| CatalogMergeConflict::FieldDefinitionDiffers { kept, discarded },
    );
    let saved_searches = union_by_id(
        &newer.saved_searches,
        &older.saved_searches,
        |it| it.id.clone(),
        conflicts,
        |kept, discarded| CatalogMergeConflict::SavedSearchDiffers { kept, discarded },
    );
    let rules = union_by_id(
        &newer.rules,
        &older.rules,
        |it| it.id.clone(),
        conflicts,
        |kept, discarded| CatalogMergeConflict::RuleDiffers { kept, discarded },
    );
    CatalogSettings {
        field_definitions,
        saved_searches,
        rules,
        correspondents: merge_correspondents(&newer.correspondents, &older.correspondents),
        layout: newer.layout.clone(),
        owner: newer_or_older(newer.owner.as_ref(), older.owner.as_ref(), conflicts, |kept, discarded| {
            CatalogMergeConflict::OwnerDiffers { kept, discarded }
        }),
        bills_tracked_since: match (newer.bills_tracked_since, older.bills_tracked_since) {
            (Some(first), Some(second)) => Some(first.min(second)),
            (first, second) => first.or(second),
        },
        dismissed_contract_proposals: union_by_key(
            &newer.dismissed_contract_proposals,
            &older.dismissed_contract_proposals,
            |name| text::lowercase(text::trim(name)),
        ),
        expectations: union_by_key(
            &newer.expectations,
            &older.expectations,
            crate::expectation::ExpectationOverride::key,
        ),
        emergency_folder: newer_or_older(
            newer.emergency_folder.as_ref(),
            older.emergency_folder.as_ref(),
            conflicts,
            |kept, discarded| CatalogMergeConflict::EmergencyFolderDiffers { kept, discarded },
        ),
    }
}

/// Contracts, cases and tax years per id: the newer copy's entries, then the older copy's it lacks.
fn merge_records(
    newer: &CatalogRecords,
    older: &CatalogRecords,
    conflicts: &mut Vec<CatalogMergeConflict>,
) -> CatalogRecords {
    CatalogRecords {
        contracts: union_reported(&newer.contracts, &older.contracts, conflicts, |kept, discarded| {
            CatalogMergeConflict::ContractDiffers { kept, discarded }
        }),
        cases: union_reported(&newer.cases, &older.cases, conflicts, |kept, discarded| {
            CatalogMergeConflict::CaseDiffers { kept, discarded }
        }),
        tax_years: union_reported(&newer.tax_years, &older.tax_years, conflicts, |kept, discarded| {
            CatalogMergeConflict::TaxYearDiffers { kept, discarded }
        }),
    }
}

/// The newer copy's entries, then the older copy's it lacks; an entry both hold differently is reported.
fn union_reported<K: Clone + Eq + Hash, V: Clone + PartialEq>(
    newer: &IndexMap<K, V>,
    older: &IndexMap<K, V>,
    conflicts: &mut Vec<CatalogMergeConflict>,
    conflict: impl Fn(V, V) -> CatalogMergeConflict,
) -> IndexMap<K, V> {
    let mut merged = newer.clone();
    for (key, value) in older {
        match newer.get(key) {
            Some(kept) if kept != value => conflicts.push(conflict(kept.clone(), value.clone())),
            Some(_) => {}
            None => {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    merged
}

/// The newer copy's value, or the older's where the newer has none; two different values are reported.
fn newer_or_older<V: Clone + PartialEq>(
    newer: Option<&V>,
    older: Option<&V>,
    conflicts: &mut Vec<CatalogMergeConflict>,
    conflict: impl Fn(V, V) -> CatalogMergeConflict,
) -> Option<V> {
    match (newer, older) {
        (Some(kept), Some(discarded)) if kept != discarded => {
            conflicts.push(conflict(kept.clone(), discarded.clone()));
            Some(kept.clone())
        }
        (Some(kept), _) => Some(kept.clone()),
        (None, other) => other.cloned(),
    }
}

/// The newer copy's entries, then the older copy's whose key the newer lacks; no conflicts.
fn union_by_key<K: Eq + Hash, V: Clone>(newer: &[V], older: &[V], key: impl Fn(&V) -> K) -> Vec<V> {
    let known: HashSet<K> = newer.iter().map(&key).collect();
    newer.iter().chain(older.iter().filter(|entry| !known.contains(&key(entry)))).cloned().collect()
}

/// The newer copy's entries in their order, then the older copy's entries it lacks; clashes reported.
fn union_by_id<K: Eq + Hash, V: Clone + PartialEq>(
    newer: &[V],
    older: &[V],
    id: impl Fn(&V) -> K,
    conflicts: &mut Vec<CatalogMergeConflict>,
    conflict: impl Fn(V, V) -> CatalogMergeConflict,
) -> Vec<V> {
    // `associateBy` keeps the last entry for a repeated id.
    let by_id: IndexMap<K, &V> = newer.iter().map(|entry| (id(entry), entry)).collect();
    let mut merged = newer.to_vec();
    for entry in older {
        match by_id.get(&id(entry)) {
            Some(kept) if *kept != entry => conflicts.push(conflict((*kept).clone(), entry.clone())),
            Some(_) => {}
            None => merged.push(entry.clone()),
        }
    }
    merged
}

/// Correspondents by name; the aliases of both copies are kept.
fn merge_correspondents(newer: &[Correspondent], older: &[Correspondent]) -> Vec<Correspondent> {
    let older_by_key: IndexMap<String, &Correspondent> = older.iter().map(|it| (it.key(), it)).collect();
    let mut merged: Vec<Correspondent> = newer
        .iter()
        .map(|correspondent| match older_by_key.get(&correspondent.key()) {
            None => correspondent.clone(),
            Some(other) => {
                let aliases: IndexSet<&String> = correspondent.aliases.iter().chain(&other.aliases).collect();
                Correspondent { aliases: aliases.into_iter().cloned().collect(), ..correspondent.clone() }
            }
        })
        .collect();
    let newer_keys: HashSet<String> = newer.iter().map(Correspondent::key).collect();
    merged.extend(older.iter().filter(|it| !newer_keys.contains(&it.key())).cloned());
    merged
}
