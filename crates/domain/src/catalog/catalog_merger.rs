use std::collections::HashSet;
use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

use crate::DomainError;
use crate::catalog::{Catalog, CatalogMergeConflict, CatalogMergeResult};
use crate::document::{Document, DocumentId};
use crate::error::require;
use crate::library::RelativePath;
use crate::link::DocumentLink;
use crate::rule::Correspondent;
use crate::settings::CatalogSettings;
use crate::taxonomy::{Category, CategoryId};

/// Merges two copies of the same catalog, for example the copies of two library folders, or a conflict copy a
/// sync client created when two devices saved at the same time.
///
/// There is no common ancestor to compare against, so an entry missing on one side cannot be told apart from an
/// entry deleted there. The merge therefore never deletes: it is the union of both copies. Where both copies
/// hold different versions of the same entry, the copy with the higher revision wins (the first one on a tie)
/// and the decision is reported as a conflict.
///
/// Links are merged as a union of pairs. Field definitions, saved searches and rules are merged per id like
/// categories; correspondents are a union by name; the newer copy's layout wins.
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
        let merged = newer.with_parts(documents, categories, tags, links, settings);
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
    }
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
