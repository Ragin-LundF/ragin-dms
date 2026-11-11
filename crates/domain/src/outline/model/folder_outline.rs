use indexmap::{IndexMap, IndexSet};

use crate::catalog::Catalog;
use crate::document::Document;
use crate::library::RelativePath;
use crate::outline::OutlineFolder;
use crate::taxonomy::CategoryId;
use crate::text;

const SAMPLE_TITLES: usize = 3;

/// The library's folder tree as the AI sees it when it suggests where to file a document. Folders the user
/// excluded (the inbox) and hidden folders are left out; a large tree is cut at the depth that keeps it within
/// the limit, and the cut folders count what they hide.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderOutline {
    pub folders: Vec<OutlineFolder>,
    pub excluded: Vec<RelativePath>,
}

impl FolderOutline {
    pub const DEFAULT_LIMIT: usize = 200;

    /// `true` if `path` is a folder of the outline.
    #[must_use]
    pub fn contains(&self, path: &RelativePath) -> bool {
        self.folders.iter().any(|folder| &folder.path == path)
    }

    /// `true` if `path` may be created as a new folder: directly below a folder of the outline or below the root,
    /// and not one of the excluded folders or inside one.
    #[must_use]
    pub fn can_create(&self, path: &RelativePath) -> bool {
        let Some(parent) = path.parent() else {
            return false;
        };
        if self.excluded.iter().any(|excluded| path == excluded || path.is_inside(excluded)) {
            return false;
        }
        parent.is_root() || self.contains(&parent)
    }

    /// The outline of `catalog` without `excluded`, cut to at most `limit` folders
    /// ([`Self::DEFAULT_LIMIT`] by default).
    #[must_use]
    pub fn of(catalog: &Catalog, excluded: Vec<RelativePath>, limit: usize) -> Self {
        let documents: Vec<&Document> = catalog
            .documents()
            .values()
            .filter(|document| !excluded.iter().any(|folder| document.path.is_inside(folder)))
            .collect();
        let all: Vec<RelativePath> = documents
            .iter()
            .flat_map(|document| ancestors_of(&document.path))
            .collect::<IndexSet<_>>()
            .into_iter()
            .filter(|folder| !excluded.iter().any(|it| folder == it || folder.is_inside(it)))
            .filter(|folder| !folder.segments().iter().any(|segment| segment.starts_with('.')))
            .collect();
        let depth = depth_within(&all, limit);
        let mut shown: Vec<&RelativePath> = all.iter().filter(|folder| folder.segments().len() <= depth).collect();
        shown.sort_by(|first, second| text::compare(first.value(), second.value()));
        shown.truncate(limit);
        let folders = shown.into_iter().map(|folder| outline_folder(folder, &documents, &all, depth)).collect();
        Self { folders, excluded }
    }
}

fn outline_folder(folder: &RelativePath, documents: &[&Document], all: &[RelativePath], depth: usize) -> OutlineFolder {
    let mut inside: Vec<&Document> =
        documents.iter().copied().filter(|document| document.path.is_inside(folder)).collect();
    let top_category = most_common_category(&inside);
    inside.sort_by_key(|document| std::cmp::Reverse(document.added_at));
    OutlineFolder {
        path: folder.clone(),
        document_count: inside.len(),
        top_category,
        sample_titles: inside.iter().take(SAMPLE_TITLES).map(|document| title_of(document)).collect(),
        hidden_subfolders: all.iter().filter(|it| it.is_inside(folder) && it.segments().len() > depth).count(),
    }
}

/// The category most documents have; on a tie the one with the greatest id.
fn most_common_category(documents: &[&Document]) -> Option<CategoryId> {
    let mut counts: IndexMap<&CategoryId, usize> = IndexMap::new();
    for id in documents.iter().filter_map(|document| document.category_id.as_ref()) {
        *counts.entry(id).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|(first_id, first), (second_id, second)| {
            first.cmp(second).then_with(|| text::compare(first_id.value(), second_id.value()))
        })
        .map(|(id, _)| id.clone())
}

/// Every folder above `path`, nearest first, without the root.
fn ancestors_of(path: &RelativePath) -> Vec<RelativePath> {
    std::iter::successors(path.parent(), RelativePath::parent).filter(|folder| !folder.is_root()).collect()
}

/// The deepest level at which the outline still has at most `limit` folders.
fn depth_within(folders: &[RelativePath], limit: usize) -> usize {
    let mut depth = folders.iter().map(|folder| folder.segments().len()).max().unwrap_or(0);
    while depth > 1 && folders.iter().filter(|folder| folder.segments().len() <= depth).count() > limit {
        depth -= 1;
    }
    depth
}

fn title_of(document: &Document) -> String {
    document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned())
}
