use std::sync::Arc;

use dms_application::port::ai::PageImage;
use dms_application::preview::{DocumentThumbnails, ThumbnailOutcome};
use dms_domain::library::{LibraryFolderId, RelativePath};

use crate::support::TestLibrary;
use crate::support::fake::ai::{FakePreviewer, PreviewRequest};
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
        let thumbnails = DocumentThumbnails::new(library.store.clone(), previewer.clone());
        Self { library, previewer, thumbnails, scan: path("letters/scan.pdf") }
    }

    fn draw(&self, folder_id: &LibraryFolderId, path: &RelativePath) -> ThumbnailOutcome {
        self.thumbnails.run(folder_id, path)
    }

    fn requested_paths(&self) -> Vec<RelativePath> {
        self.previewer.requests.lock().iter().map(|request| request.path.clone()).collect()
    }
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
