use std::sync::Arc;

use dms_application::port::ai::PageImage;
use dms_application::preview::{DocumentThumbnails, ThumbnailOutcome};
use dms_domain::document::Document;
use dms_domain::library::{LibraryFolderId, RelativePath};

use crate::support::TestLibrary;
use crate::support::fake::ai::{FakePreviewer, PreviewRequest};
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::path;

const CAPACITY: usize = 200;

struct Kit {
    library: TestLibrary,
    previewer: Arc<FakePreviewer>,
    thumbnails: DocumentThumbnails,
    scan: RelativePath,
}

impl Kit {
    fn new() -> Self {
        let library = TestLibrary::new();
        let previewer = Arc::new(FakePreviewer::default());
        let thumbnails = thumbnails_of(&library, &previewer);
        Self { library, previewer, thumbnails, scan: path("letters/scan.pdf") }
    }

    /// The same library and files with empty memory, as after a restart.
    fn restarted(self) -> Self {
        let thumbnails = thumbnails_of(&self.library, &self.previewer);
        self.previewer.requests.set(Vec::new());
        Self { thumbnails, ..self }
    }

    /// A catalogued document of `size` bytes whose picture is the JPEG `bytes`.
    fn document_with_picture(&self, size: usize, bytes: &[u8]) -> Document {
        let document = self.library.add("letters/invoice.pdf", &"x".repeat(size));
        self.previewer.pictures.update(|pictures| pictures.insert(document.path.clone(), jpeg(bytes)));
        document
    }

    fn draw(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> ThumbnailOutcome {
        self.thumbnails.run(folder_id, path)
    }

    fn requested_paths(&self) -> Vec<RelativePath> {
        self.previewer.requests.lock().iter().map(|request| request.path.clone()).collect()
    }
}

fn thumbnails_of(library: &TestLibrary, previewer: &Arc<FakePreviewer>) -> DocumentThumbnails {
    DocumentThumbnails::new(library.store.clone(), previewer.clone(), library.previews.clone())
}

fn jpeg(bytes: &[u8]) -> PageImage {
    PageImage { bytes: bytes.to_vec(), mime_type: "image/jpeg".to_owned() }
}

fn picture() -> PageImage {
    PageImage { bytes: vec![1, 2, 3], mime_type: "image/png".to_owned() }
}

fn rendered_bytes(outcome: ThumbnailOutcome) -> Vec<u8> {
    match outcome {
        ThumbnailOutcome::Rendered(image) => image.bytes,
        other => panic!("expected Rendered, got {other:?}"),
    }
}

fn pages() -> Vec<RelativePath> {
    (0..=CAPACITY).map(|index| path(&format!("scans/page-{index}.pdf"))).collect()
}

#[test]
fn renders_the_picture_of_a_file_in_a_library_folder_at_thumbnail_size() {
    let kit = Kit::new();
    kit.library.open();
    kit.previewer.pictures.update(|pictures| pictures.insert(kit.scan.clone(), picture()));

    let outcome = kit.draw(&kit.library.sync_a.id, &kit.scan);

    assert_eq!(rendered_bytes(outcome), picture().bytes);
    assert_eq!(
        kit.previewer.requests.get(),
        vec![PreviewRequest { folder: kit.library.sync_a.clone(), path: kit.scan.clone(), max_side: 320 }]
    );
}

#[test]
fn a_file_without_a_picture_has_none() {
    let kit = Kit::new();
    kit.library.open();

    let outcome = kit.draw(&kit.library.primary.id, &kit.scan);

    assert!(matches!(outcome, ThumbnailOutcome::NoPicture), "got {outcome:?}");
}

#[test]
fn nothing_is_rendered_while_no_library_is_open() {
    let kit = Kit::new();
    kit.previewer.pictures.update(|pictures| pictures.insert(kit.scan.clone(), picture()));

    let outcome = kit.draw(&kit.library.primary.id, &kit.scan);

    assert!(matches!(outcome, ThumbnailOutcome::FolderNotFound), "got {outcome:?}");
    assert_eq!(kit.previewer.requests.get(), Vec::new());
}

#[test]
fn a_folder_that_is_not_part_of_the_library_is_not_found() {
    let kit = Kit::new();
    kit.library.open();
    kit.previewer.pictures.update(|pictures| pictures.insert(kit.scan.clone(), picture()));

    let outcome = kit.draw(&LibraryFolderId::new("elsewhere").expect("valid id"), &kit.scan);

    assert!(matches!(outcome, ThumbnailOutcome::FolderNotFound), "got {outcome:?}");
    assert_eq!(kit.previewer.requests.get(), Vec::new());
}

#[test]
fn a_file_drawn_again_comes_from_memory() {
    let kit = Kit::new();
    kit.library.open();
    kit.previewer.pictures.update(|pictures| pictures.insert(kit.scan.clone(), picture()));

    kit.draw(&kit.library.primary.id, &kit.scan);
    let again = kit.draw(&kit.library.primary.id, &kit.scan);

    assert_eq!(rendered_bytes(again), picture().bytes);
    assert_eq!(kit.previewer.requests.lock().len(), 1);
}

#[test]
fn a_file_without_a_picture_is_remembered_as_such() {
    let kit = Kit::new();
    kit.library.open();

    kit.draw(&kit.library.primary.id, &kit.scan);
    let again = kit.draw(&kit.library.primary.id, &kit.scan);

    assert!(matches!(again, ThumbnailOutcome::NoPicture), "got {again:?}");
    assert_eq!(kit.previewer.requests.lock().len(), 1);
}

#[test]
fn the_same_path_in_another_folder_is_rendered_on_its_own() {
    let kit = Kit::new();
    kit.library.open();
    kit.previewer.pictures.update(|pictures| pictures.insert(kit.scan.clone(), picture()));

    kit.draw(&kit.library.primary.id, &kit.scan);
    kit.draw(&kit.library.sync_b.id, &kit.scan);

    let folders: Vec<_> = kit.previewer.requests.lock().iter().map(|request| request.folder.clone()).collect();
    assert_eq!(folders, vec![kit.library.primary.clone(), kit.library.sync_b.clone()]);
}

#[test]
fn the_least_recently_drawn_file_is_forgotten_once_the_memory_is_full() {
    let kit = Kit::new();
    kit.library.open();
    let paths = pages();
    let primary = kit.library.primary.id.clone();

    for page in &paths {
        kit.draw(&primary, page);
    }
    kit.previewer.requests.set(Vec::new());
    kit.draw(&primary, paths.last().expect("pages"));
    kit.draw(&primary, &paths[0]);

    assert_eq!(kit.requested_paths(), vec![paths[0].clone()]);
}

#[test]
fn drawing_a_file_again_keeps_it_in_memory_longer_than_files_drawn_later() {
    let kit = Kit::new();
    kit.library.open();
    let paths = pages();
    let primary = kit.library.primary.id.clone();

    for page in &paths[..paths.len() - 1] {
        kit.draw(&primary, page);
    }
    kit.draw(&primary, &paths[0]);
    kit.draw(&primary, paths.last().expect("pages"));
    kit.previewer.requests.set(Vec::new());
    kit.draw(&primary, &paths[0]);
    kit.draw(&primary, &paths[1]);

    assert_eq!(kit.requested_paths(), vec![paths[1].clone()]);
}

#[test]
fn the_preview_of_a_document_is_stored_in_every_folder_for_its_content_hash() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.document_with_picture(100, &[7, 8, 9]);

    kit.draw(&kit.library.sync_a.id, &document.path);

    for folder in kit.library.all_folders() {
        let stored = kit.library.previews.stored(&folder, &document.id);
        assert_eq!(stored, Some((document.content_hash.clone(), vec![7, 8, 9])), "{}", folder.display_name);
    }
}

#[test]
fn a_stored_preview_is_shown_after_a_restart_without_rendering_the_file() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.document_with_picture(100, &[7, 8, 9]);
    kit.draw(&kit.library.primary.id, &document.path);
    let kit = kit.restarted();

    let outcome = kit.draw(&kit.library.sync_b.id, &document.path);

    assert_eq!(rendered_bytes(outcome), vec![7, 8, 9]);
    assert_eq!(kit.previewer.requests.get(), Vec::new());
}

#[test]
fn a_preview_of_a_tenth_of_the_documents_size_or_more_is_kept_in_memory_only() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.document_with_picture(30, &[7, 8, 9]);

    let first = kit.draw(&kit.library.primary.id, &document.path);
    let again = kit.draw(&kit.library.primary.id, &document.path);

    assert_eq!(rendered_bytes(first), vec![7, 8, 9]);
    assert_eq!(rendered_bytes(again), vec![7, 8, 9]);
    assert_eq!(kit.previewer.requests.lock().len(), 1);
    assert_eq!(kit.library.previews.entries.get(), std::collections::HashMap::new());
}

#[test]
fn a_picture_that_is_no_jpeg_is_not_stored() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.library.add("letters/invoice.pdf", &"x".repeat(100));
    kit.previewer.pictures.update(|pictures| pictures.insert(document.path.clone(), picture()));

    let outcome = kit.draw(&kit.library.primary.id, &document.path);

    assert_eq!(rendered_bytes(outcome), picture().bytes);
    assert_eq!(kit.library.previews.entries.get(), std::collections::HashMap::new());
}

#[test]
fn a_changed_document_is_drawn_again_and_its_new_preview_replaces_the_old_one() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.document_with_picture(100, &[7, 8, 9]);
    kit.draw(&kit.library.primary.id, &document.path);
    kit.previewer.pictures.update(|pictures| pictures.insert(document.path.clone(), jpeg(&[4, 5, 6])));
    let changed = FakeFolders::hash_of("changed content");
    let _ = kit.library.store.commit(|catalog| Ok(catalog.update_content(&document.id, changed.clone(), 200)));

    let outcome = kit.draw(&kit.library.primary.id, &document.path);

    assert_eq!(rendered_bytes(outcome), vec![4, 5, 6]);
    assert_eq!(kit.previewer.requests.lock().len(), 2);
    for folder in kit.library.all_folders() {
        let stored = kit.library.previews.stored(&folder, &document.id);
        assert_eq!(stored, Some((changed.clone(), vec![4, 5, 6])), "{}", folder.display_name);
    }
}

#[test]
fn an_offline_sync_folder_gets_no_preview_and_no_journal_entry() {
    let kit = Kit::new();
    kit.library.open();
    let document = kit.document_with_picture(100, &[7, 8, 9]);
    kit.library.folders.make_unreachable(&kit.library.sync_b);

    kit.draw(&kit.library.primary.id, &document.path);

    assert_eq!(kit.library.previews.stored(&kit.library.sync_b, &document.id), None);
    assert!(kit.library.previews.stored(&kit.library.sync_a, &document.id).is_some());
    assert_eq!(kit.library.journal.operations.get().len(), 0);
}
