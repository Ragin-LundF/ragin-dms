//! The folder tree and the category tree as indented lists (Kotlin `LibraryTrees`).

use std::collections::{HashMap, HashSet};

use dms_domain::document::Document;
use dms_domain::library::RelativePath;
use dms_domain::taxonomy::{Category, CategoryId};
use dms_domain::text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    pub path: RelativePath,
    pub name: String,
    pub depth: usize,
    /// Every document at or below the folder.
    pub document_count: usize,
    pub has_children: bool,
    pub expanded: bool,
}

#[derive(Debug, Clone)]
pub struct CategoryEntry<'a> {
    pub category: &'a Category,
    pub depth: usize,
    /// The documents with exactly this category.
    pub document_count: usize,
}

/// The root ("All documents") first, then every folder of `documents` and `folders_on_disk` in path order, each
/// followed by its subfolders while it is in `expanded`; `None` unfolds everything.
pub fn folders<'a>(
    documents: impl IntoIterator<Item = &'a Document>,
    folders_on_disk: &[RelativePath],
    expanded: Option<&HashSet<RelativePath>>,
) -> Vec<FolderEntry> {
    let mut counts: HashMap<RelativePath, usize> = HashMap::new();
    let mut total = 0;
    for document in documents {
        total += 1;
        for folder in ancestors_of(&document.path) {
            *counts.entry(folder).or_default() += 1;
        }
    }
    let mut known: HashSet<RelativePath> = counts.keys().cloned().collect();
    known.extend(folders_on_disk.iter().flat_map(self_and_ancestors));
    // Sorting by segments keeps every folder directly above its subfolders.
    let mut sorted: Vec<(String, RelativePath)> =
        known.into_iter().map(|path| (text::lowercase(&path.segments().join("\u{0}")), path)).collect();
    sorted.sort_by(|first, second| text::compare(&first.0, &second.0));
    let sorted: Vec<RelativePath> = sorted.into_iter().map(|(_, path)| path).collect();
    let root = FolderEntry {
        path: RelativePath::root(),
        name: "All documents".to_owned(),
        depth: 0,
        document_count: total,
        has_children: !sorted.is_empty(),
        expanded: true,
    };
    let entries = std::iter::once(root).chain(sorted.iter().enumerate().map(|(index, folder)| FolderEntry {
        path: folder.clone(),
        name: folder.file_name().to_owned(),
        depth: folder.segments().len(),
        document_count: counts.get(folder).copied().unwrap_or(0),
        has_children: sorted.get(index + 1).is_some_and(|next| next.is_inside(folder)),
        expanded: expanded.is_none_or(|open| open.contains(folder)),
    }));
    visible(entries, |entry| entry.depth, |entry| entry.has_children && !entry.expanded)
}

/// Top-level categories by name, each followed by all its subcategories.
pub fn categories<'a, 'd>(
    categories: impl IntoIterator<Item = &'a Category>,
    documents: impl IntoIterator<Item = &'d Document>,
) -> Vec<CategoryEntry<'a>> {
    let mut children: HashMap<Option<&CategoryId>, Vec<&'a Category>> = HashMap::new();
    for category in categories {
        children.entry(category.parent_id.as_ref()).or_default().push(category);
    }
    let mut counts: HashMap<&CategoryId, usize> = HashMap::new();
    for document in documents {
        if let Some(id) = &document.category_id {
            *counts.entry(id).or_default() += 1;
        }
    }
    let mut entries = Vec::new();
    flatten(None, 0, &children, &counts, &mut entries, &mut HashSet::new());
    entries
}

fn flatten<'a>(
    parent: Option<&CategoryId>,
    depth: usize,
    children: &HashMap<Option<&CategoryId>, Vec<&'a Category>>,
    counts: &HashMap<&CategoryId, usize>,
    entries: &mut Vec<CategoryEntry<'a>>,
    visited: &mut HashSet<CategoryId>,
) {
    let Some(below) = children.get(&parent) else {
        return;
    };
    let mut sorted: Vec<(String, &'a Category)> =
        below.iter().map(|category| (text::lowercase(&category.name), *category)).collect();
    sorted.sort_by(|first, second| text::compare(&first.0, &second.0));
    for (_, category) in sorted {
        // A cycle in a damaged catalog would recurse forever; Kotlin never reaches one from the top level either.
        if !visited.insert(category.id.clone()) {
            continue;
        }
        entries.push(CategoryEntry { category, depth, document_count: counts.get(&category.id).copied().unwrap_or(0) });
        flatten(Some(&category.id), depth + 1, children, counts, entries, visited);
    }
}

/// Drops every entry below a folded one; entries are in tree order, children right after their parent.
fn visible<T>(entries: impl Iterator<Item = T>, depth_of: impl Fn(&T) -> usize, folded: impl Fn(&T) -> bool) -> Vec<T> {
    let mut hidden_below = usize::MAX;
    entries
        .filter(|entry| {
            let depth = depth_of(entry);
            if depth > hidden_below {
                return false;
            }
            hidden_below = if folded(entry) { depth } else { usize::MAX };
            true
        })
        .collect()
}

/// The folders containing `path`, without the root.
pub fn ancestors_of(path: &RelativePath) -> Vec<RelativePath> {
    path.parent().map(|parent| self_and_ancestors(&parent)).unwrap_or_default()
}

fn self_and_ancestors(folder: &RelativePath) -> Vec<RelativePath> {
    if folder.is_root() {
        return Vec::new();
    }
    let mut all = vec![folder.clone()];
    all.extend(ancestors_of(folder));
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, path};

    #[test]
    fn folders_list_parents_above_children_with_counts_and_empty_disk_folders() {
        let catalog = TestCatalog::new()
            .document("b/x.pdf", |_| {})
            .document("a/c/y.pdf", |_| {})
            .document("A2/z.pdf", |_| {})
            .document("top.pdf", |_| {})
            .build();
        let entries = folders(catalog.documents().values(), &[path("a/empty")], None);
        let paths: Vec<&str> = entries.iter().map(|entry| entry.path.value()).collect();
        assert_eq!(paths, ["", "a", "a/c", "a/empty", "A2", "b"]);
        assert_eq!(entries[0].document_count, 4);
        assert_eq!(entries[1].document_count, 1);
        assert!(entries[1].has_children);
        assert!(!entries[3].has_children);
        assert_eq!(entries[2].depth, 2);
    }

    #[test]
    fn folded_folders_hide_their_subfolders_but_top_level_folders_always_show() {
        let catalog = TestCatalog::new().document("a/b/c/x.pdf", |_| {}).document("d/y.pdf", |_| {}).build();
        let expanded = HashSet::from([path("a")]);
        let entries = folders(catalog.documents().values(), &[], Some(&expanded));
        let paths: Vec<&str> = entries.iter().map(|entry| entry.path.value()).collect();
        assert_eq!(paths, ["", "a", "a/b", "d"]);
    }

    #[test]
    fn categories_are_sorted_by_name_with_children_below_their_parent() {
        let catalog = TestCatalog::new()
            .category("c-z", "Zeta", None)
            .category("c-a", "alpha", None)
            .category("c-b", "Beta", Some("c-z"))
            .document("x.pdf", |document| {
                document.category("c-b");
            })
            .build();
        let entries = categories(catalog.categories().values(), catalog.documents().values());
        let names: Vec<(&str, usize, usize)> =
            entries.iter().map(|entry| (entry.category.name.as_str(), entry.depth, entry.document_count)).collect();
        assert_eq!(names, [("alpha", 0, 0), ("Zeta", 0, 0), ("Beta", 1, 1)]);
    }
}
