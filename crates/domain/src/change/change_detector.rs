use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::catalog::Catalog;
use crate::change::{
    ContentConflict, DetectedChange, FileFacts, FolderSnapshot, MissingFile, ModifiedFile, MovedFile, NewFile,
};
use crate::document::{Document, DocumentId};
use crate::library::{LibraryFolderId, RelativePath};
use crate::text;

/// Compares the catalog with snapshots of the reachable library folders and reports every difference.
///
/// Detection only: nothing here decides what to do about a change. Folders without a snapshot (for example an
/// offline network share) are not reported as missing anything.
pub struct ChangeDetector;

/// How one document relates to one folder.
enum Comparison {
    Unchanged,
    Missing,
    Changed(DetectedChange),
}

type FoldersByDocument = IndexMap<DocumentId, IndexSet<LibraryFolderId>>;

impl ChangeDetector {
    #[must_use]
    pub fn detect(catalog: &Catalog, snapshots: &[FolderSnapshot]) -> Vec<DetectedChange> {
        let known_paths: HashSet<&RelativePath> = catalog.documents().values().map(|document| &document.path).collect();
        let mut missing_in = FoldersByDocument::new();
        let mut present_in = FoldersByDocument::new();
        let mut untracked: IndexMap<RelativePath, IndexMap<LibraryFolderId, FileFacts>> = IndexMap::new();
        let mut changes = Vec::new();

        for snapshot in snapshots {
            let mut unclaimed: IndexMap<RelativePath, FileFacts> = snapshot
                .files
                .iter()
                .filter(|(path, _)| !known_paths.contains(path))
                .map(|(path, facts)| (path.clone(), facts.clone()))
                .collect();
            for document in catalog.documents().values() {
                match compare(document, snapshot, &mut unclaimed) {
                    Comparison::Unchanged => add_folder(&mut present_in, document, snapshot),
                    Comparison::Missing => add_folder(&mut missing_in, document, snapshot),
                    Comparison::Changed(change) => changes.push(change),
                }
            }
            for (path, facts) in unclaimed {
                untracked.entry(path).or_default().insert(snapshot.folder_id.clone(), facts);
            }
        }

        changes.extend(missing_files(catalog, missing_in, &present_in));
        changes.extend(untracked.into_iter().map(|(path, facts)| untracked_change(path, facts)));
        changes.sort_by(|first, second| text::compare(first.path().value(), second.path().value()));
        changes
    }
}

/// Compares one document with one folder. A move claims the untracked file it matched.
fn compare(
    document: &Document,
    snapshot: &FolderSnapshot,
    unclaimed: &mut IndexMap<RelativePath, FileFacts>,
) -> Comparison {
    if let Some(facts) = snapshot.files.get(&document.path) {
        return modification(document, facts, &snapshot.folder_id).map_or(Comparison::Unchanged, Comparison::Changed);
    }
    let new_path = unclaimed
        .iter()
        .find(|(_, facts)| facts.content_hash.as_ref() == Some(&document.content_hash))
        .map(|(path, _)| path.clone());
    let Some(new_path) = new_path else {
        return Comparison::Missing;
    };
    unclaimed.shift_remove(&new_path);
    Comparison::Changed(DetectedChange::MovedFile(MovedFile {
        document_id: document.id.clone(),
        path: new_path,
        previous_path: document.path.clone(),
        folder_id: snapshot.folder_id.clone(),
    }))
}

fn modification(document: &Document, facts: &FileFacts, folder_id: &LibraryFolderId) -> Option<DetectedChange> {
    let changed = match &facts.content_hash {
        Some(hash) => hash != &document.content_hash,
        None => facts.size != document.size,
    };
    changed.then(|| {
        DetectedChange::ModifiedFile(ModifiedFile {
            document_id: document.id.clone(),
            path: document.path.clone(),
            folder_id: folder_id.clone(),
            size: facts.size,
            content_hash: facts.content_hash.clone(),
        })
    })
}

fn add_folder(folders: &mut FoldersByDocument, document: &Document, snapshot: &FolderSnapshot) {
    folders.entry(document.id.clone()).or_default().insert(snapshot.folder_id.clone());
}

fn missing_files(
    catalog: &Catalog,
    missing_in: FoldersByDocument,
    present_in: &FoldersByDocument,
) -> Vec<DetectedChange> {
    missing_in
        .into_iter()
        .filter_map(|(document_id, folders)| {
            let path = catalog.documents().get(&document_id)?.path.clone();
            let present = present_in.get(&document_id).cloned().unwrap_or_default();
            Some(DetectedChange::MissingFile(MissingFile {
                document_id,
                path,
                missing_in: folders,
                present_in: present,
            }))
        })
        .collect()
}

fn untracked_change(path: RelativePath, facts_by_folder: IndexMap<LibraryFolderId, FileFacts>) -> DetectedChange {
    let hashes: IndexMap<_, _> = facts_by_folder
        .iter()
        .filter_map(|(folder_id, facts)| facts.content_hash.clone().map(|hash| (folder_id.clone(), hash)))
        .collect();
    if hashes.values().collect::<HashSet<_>>().len() > 1 {
        return DetectedChange::ContentConflict(ContentConflict { path, content_hashes: hashes });
    }
    let size = facts_by_folder.values().next().map_or(0, |facts| facts.size);
    DetectedChange::NewFile(NewFile {
        path,
        present_in: facts_by_folder.into_keys().collect(),
        size,
        content_hash: hashes.into_values().next(),
    })
}
