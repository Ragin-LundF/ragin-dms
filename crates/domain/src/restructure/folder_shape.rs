use std::collections::HashSet;
use std::hash::Hash;

use chrono::Datelike;
use indexmap::IndexMap;

use crate::catalog::Catalog;
use crate::document::Document;
use crate::library::RelativePath;
use crate::restructure::FlatFolder;
use crate::settings::InboxSettings;
use crate::text;

const MIN_SPLIT: usize = 10;
const MIN_GROUPS: usize = 2;

/// Finds folders that would profit from a restructure: folders without subfolders that hold more than
/// [`FolderShape::DEFAULT_THRESHOLD`] documents, or at least 10 documents that split cleanly into several
/// categories or years. The inbox is never one of them.
pub struct FolderShape;

impl FolderShape {
    pub const DEFAULT_THRESHOLD: usize = 40;

    #[must_use]
    pub fn analyse(catalog: &Catalog, inbox: &InboxSettings, threshold: usize) -> Vec<FlatFolder> {
        let documents: Vec<&Document> =
            catalog.documents().values().filter(|document| !inbox.contains(&document.path)).collect();
        let mut by_folder: IndexMap<RelativePath, Vec<&Document>> = IndexMap::new();
        for document in &documents {
            by_folder.entry(document.path.parent().unwrap_or_default()).or_default().push(document);
        }
        let with_subfolders: HashSet<RelativePath> = documents
            .iter()
            .flat_map(|document| {
                std::iter::successors(document.path.parent().and_then(|parent| parent.parent()), RelativePath::parent)
            })
            .collect();
        let mut flat: Vec<FlatFolder> = by_folder
            .into_iter()
            .filter(|(folder, _)| !(with_subfolders.contains(folder) || (inbox.enabled && folder == &inbox.path)))
            .filter(|(_, inside)| inside.len() > threshold || splits_cleanly(inside))
            .map(|(path, inside)| FlatFolder { path, file_count: inside.len() })
            .collect();
        flat.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        flat
    }
}

fn splits_cleanly(documents: &[&Document]) -> bool {
    if documents.len() < MIN_SPLIT {
        return false;
    }
    let categories: Vec<_> = documents.iter().map(|document| document.category_id.as_ref()).collect();
    let years: Vec<_> =
        documents.iter().map(|document| document.metadata.document_date.map(|date| date.year())).collect();
    (categories.iter().all(Option::is_some) && distinct_count(&categories) >= MIN_GROUPS)
        || (years.iter().all(Option::is_some) && distinct_count(&years) >= MIN_GROUPS)
}

fn distinct_count<T: Eq + Hash>(values: &[T]) -> usize {
    values.iter().collect::<HashSet<_>>().len()
}
