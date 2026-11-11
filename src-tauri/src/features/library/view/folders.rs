//! The folder view: what a folder holds, derived from the catalog and the folders on disk, and its words
//! (Kotlin `FolderContents`, `FolderTexts`, `FolderBrowser.derive`).

use std::collections::HashSet;

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::library::RelativePath;
use dms_domain::text;

use super::state::{count, rows_of};
use super::trees::{self, FolderEntry};
use crate::features::library::dto::folders::{
    LibraryConfirmationDto, LibraryCrumbDto, LibraryFolderEntryDto, LibraryFolderTileDto, LibraryFolderViewDto,
};

const NEW_FOLDER_NAME: &str = "New folder";
const LIBRARY_NAME: &str = "Library";

/// What the folder view asks for, parsed.
pub struct FolderViewParams {
    pub open: RelativePath,
    pub expanded: HashSet<RelativePath>,
    pub query: String,
    pub folders_on_disk: Vec<RelativePath>,
}

pub fn present_folder_view(catalog: &Catalog, params: &FolderViewParams) -> LibraryFolderViewDto {
    let documents = catalog.documents();
    let folders = known_folders(catalog, &params.folders_on_disk);
    let mut tree = trees::folders(documents.values(), &params.folders_on_disk, Some(&params.expanded));
    tree.remove(0);
    let inside = documents_in(&params.open, documents.values(), &params.query);
    LibraryFolderViewDto {
        crumbs: crumbs(&params.open),
        tree: tree.into_iter().map(entry_dto).collect(),
        library_count: count(documents.len()),
        tiles: children_of(&params.open, &folders)
            .into_iter()
            .map(|folder| {
                let document_count = documents.values().filter(|document| document.path.is_inside(folder)).count();
                let folder_count = children_of(folder, &folders).len();
                LibraryFolderTileDto {
                    path: folder.value().to_owned(),
                    name: folder.file_name().to_owned(),
                    caption: tile_caption(document_count, folder_count),
                }
            })
            .collect(),
        documents: rows_of(&inside, catalog),
        search_placeholder: format!("Search in {}…", quoted_or_top_level(&params.open)),
        empty_text: format!(
            "No documents directly in {}. Drop files here to add them.",
            quoted_or_top_level(&params.open)
        ),
    }
}

/// The rows of a "Move to…" picker below its "Library" row: every folder, also empty ones on disk.
pub fn picker_rows(
    catalog: &Catalog,
    folders_on_disk: &[RelativePath],
    expanded: &HashSet<RelativePath>,
) -> Vec<LibraryFolderEntryDto> {
    let mut rows = trees::folders(catalog.documents().values(), folders_on_disk, Some(expanded));
    rows.remove(0);
    rows.into_iter().map(entry_dto).collect()
}

/// "New folder", or "New folder 2" and so on while a folder in `parent` already has that name.
pub fn proposed_name(catalog: &Catalog, folders_on_disk: &[RelativePath], parent: &RelativePath) -> String {
    let folders = known_folders(catalog, folders_on_disk);
    let taken: HashSet<String> =
        children_of(parent, &folders).iter().map(|folder| text::lowercase(folder.file_name())).collect();
    let mut number = 1;
    loop {
        let name = if number == 1 { NEW_FOLDER_NAME.to_owned() } else { format!("{NEW_FOLDER_NAME} {number}") };
        if !taken.contains(&text::lowercase(&name)) {
            return name;
        }
        number += 1;
    }
}

/// The question before deleting `folder`, from what really lies in it on disk (`files`, `subfolders`): its
/// documents, the files the library does not know and the subfolders go to the trash.
pub fn delete_confirmation(
    catalog: &Catalog,
    folder: &RelativePath,
    files: &[RelativePath],
    subfolders: &[RelativePath],
    folders_on_disk: &[RelativePath],
) -> LibraryConfirmationDto {
    let mut below: HashSet<&RelativePath> = subfolders.iter().collect();
    let known = known_folders(catalog, folders_on_disk);
    below.extend(known.iter().filter(|known| known.is_inside(folder)));
    let inside: Vec<&Document> =
        catalog.documents().values().filter(|document| document.path.is_inside(folder)).collect();
    let catalogued: HashSet<&RelativePath> = inside.iter().map(|document| &document.path).collect();
    let mut others: Vec<&RelativePath> = files.iter().filter(|file| !catalogued.contains(file)).collect();
    others.sort_by(|first, second| text::compare(first.value(), second.value()));
    LibraryConfirmationDto {
        title: format!("Delete \"{}\"?", folder.file_name()),
        message: delete_message(inside.len(), others.len(), below.len()),
        affected: inside
            .iter()
            .map(|document| document.path.value().to_owned())
            .chain(others.iter().map(|file| format!("{} (not in the library)", file.value())))
            .collect(),
        confirm_label: "Delete folder".to_owned(),
    }
}

/// Every folder of the library without the top level, in tree order: of the documents and on disk.
pub fn known_folders(catalog: &Catalog, folders_on_disk: &[RelativePath]) -> Vec<RelativePath> {
    trees::folders(catalog.documents().values(), folders_on_disk, None)
        .into_iter()
        .map(|entry| entry.path)
        .filter(|path| !path.is_root())
        .collect()
}

pub fn children_of<'a>(parent: &RelativePath, folders: &'a [RelativePath]) -> Vec<&'a RelativePath> {
    folders.iter().filter(|folder| folder.parent().unwrap_or_else(RelativePath::root) == *parent).collect()
}

fn crumbs(open: &RelativePath) -> Vec<LibraryCrumbDto> {
    let mut path = Vec::new();
    let mut current = Some(open.clone()).filter(|folder| !folder.is_root());
    while let Some(folder) = current {
        current = folder.parent().filter(|parent| !parent.is_root());
        path.push(folder);
    }
    path.reverse();
    std::iter::once(LibraryCrumbDto { path: String::new(), name: LIBRARY_NAME.to_owned() })
        .chain(
            path.into_iter()
                .map(|folder| LibraryCrumbDto { name: folder.file_name().to_owned(), path: folder.value().to_owned() }),
        )
        .collect()
}

/// The documents directly in `open` whose title or file name contains `query`, newest first.
fn documents_in<'a>(
    open: &RelativePath,
    documents: impl Iterator<Item = &'a Document>,
    query: &str,
) -> Vec<&'a Document> {
    let needle = text::lowercase(text::trim(query));
    let mut found: Vec<&Document> = documents
        .filter(|document| document.path.parent().unwrap_or_else(RelativePath::root) == *open)
        .filter(|document| {
            needle.is_empty()
                || document.metadata.title.as_deref().is_some_and(|title| text::lowercase(title).contains(&needle))
                || text::lowercase(document.path.file_name()).contains(&needle)
        })
        .collect();
    found.sort_by(|first, second| {
        second.metadata.document_date.cmp(&first.metadata.document_date).then_with(|| {
            text::compare(&text::lowercase(first.path.file_name()), &text::lowercase(second.path.file_name()))
        })
    });
    found
}

fn delete_message(documents: usize, other_files: usize, subfolders: usize) -> String {
    if documents == 0 && other_files == 0 && subfolders == 0 {
        return "The folder is empty. It is removed from every library folder.".to_owned();
    }
    let parts: Vec<String> = [
        (documents > 0).then(|| counted(documents, "document")),
        (other_files > 0).then(|| format!("{} not in the library", counted(other_files, "other file"))),
        (subfolders > 0).then(|| counted(subfolders, "subfolder")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let verb = if documents + other_files + subfolders == 1 { "goes" } else { "go" };
    let listed = match parts.as_slice() {
        [single] => single.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        [] => String::new(),
    };
    format!("{listed} inside {verb} to the trash, in every library folder.")
}

fn tile_caption(documents: usize, folders: usize) -> String {
    let parts: Vec<String> =
        [(documents > 0).then(|| counted(documents, "document")), (folders > 0).then(|| counted(folders, "folder"))]
            .into_iter()
            .flatten()
            .collect();
    if parts.is_empty() { "Empty".to_owned() } else { parts.join(" · ") }
}

fn quoted_or_top_level(folder: &RelativePath) -> String {
    if folder.is_root() { "the top level".to_owned() } else { format!("\"{}\"", folder.file_name()) }
}

fn counted(count: usize, singular: &str) -> String {
    if count == 1 { format!("1 {singular}") } else { format!("{count} {singular}s") }
}

fn entry_dto(entry: FolderEntry) -> LibraryFolderEntryDto {
    LibraryFolderEntryDto {
        path: entry.path.value().to_owned(),
        name: entry.name,
        depth: count(entry.depth),
        document_count: count(entry.document_count),
        has_children: entry.has_children,
        expanded: entry.expanded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date, path};

    fn sample() -> Catalog {
        TestCatalog::new()
            .document("Home/old.pdf", |document| {
                document.dated(date(2020, 1, 1));
            })
            .document("Home/new.pdf", |document| {
                document.dated(date(2025, 1, 1)).title("Lease");
            })
            .document("Home/Power/bill.pdf", |_| {})
            .build()
    }

    fn params(open: &str, query: &str) -> FolderViewParams {
        FolderViewParams {
            open: path(open),
            expanded: HashSet::new(),
            query: query.to_owned(),
            folders_on_disk: vec![path("Home/Empty")],
        }
    }

    #[test]
    fn the_open_folder_shows_crumbs_tiles_and_its_own_documents_newest_first() {
        let view = present_folder_view(&sample(), &params("Home", ""));
        assert_eq!(view.crumbs.iter().map(|crumb| crumb.name.as_str()).collect::<Vec<_>>(), ["Library", "Home"]);
        assert_eq!(
            view.tiles.iter().map(|tile| (tile.name.as_str(), tile.caption.as_str())).collect::<Vec<_>>(),
            [("Empty", "Empty"), ("Power", "1 document")]
        );
        assert_eq!(view.documents.iter().map(|row| row.file_name.as_str()).collect::<Vec<_>>(), ["new.pdf", "old.pdf"]);
        assert_eq!(view.search_placeholder, "Search in \"Home\"…");
        assert_eq!(view.tree.iter().map(|entry| entry.path.as_str()).collect::<Vec<_>>(), ["Home"]);
    }

    #[test]
    fn the_search_matches_title_or_file_name() {
        let view = present_folder_view(&sample(), &params("Home", "LEASE"));
        assert_eq!(view.documents.len(), 1);
        assert_eq!(
            present_folder_view(&sample(), &params("", "")).empty_text,
            "No documents directly in the top level. Drop files here to add them."
        );
    }

    #[test]
    fn proposed_names_skip_taken_ones_ignoring_case() {
        let catalog = TestCatalog::new().document("new folder/a.pdf", |_| {}).build();
        assert_eq!(proposed_name(&catalog, &[path("New folder 2")], &path("")), "New folder 3");
        assert_eq!(proposed_name(&catalog, &[], &path("new folder")), "New folder");
    }

    #[test]
    fn delete_messages_name_what_goes_along() {
        assert_eq!(delete_message(0, 0, 0), "The folder is empty. It is removed from every library folder.");
        assert_eq!(delete_message(1, 0, 0), "1 document inside goes to the trash, in every library folder.");
        assert_eq!(
            delete_message(2, 1, 3),
            "2 documents, 1 other file not in the library and 3 subfolders inside go to the trash, in every library folder."
        );
        let confirmation = delete_confirmation(
            &sample(),
            &path("Home/Power"),
            &[path("Home/Power/bill.pdf"), path("Home/Power/x.txt")],
            &[],
            &[],
        );
        assert_eq!(confirmation.affected, ["Home/Power/bill.pdf", "Home/Power/x.txt (not in the library)"]);
        assert_eq!(confirmation.title, "Delete \"Power\"?");
        let home = delete_confirmation(&sample(), &path("Home"), &[], &[path("Home/Power")], &[path("Home/Empty")]);
        assert_eq!(home.message, "3 documents and 2 subfolders inside go to the trash, in every library folder.");
    }
}
