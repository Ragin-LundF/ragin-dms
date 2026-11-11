#![allow(clippy::default_trait_access)] // IndexSet is built without naming indexmap, not a dependency here
#![allow(clippy::single_range_in_vec_init)] // a list of page ranges that holds one range

use chrono::{DateTime, Duration, Utc};
use dms_application::common::OperationOutcome;
use dms_application::pdf::SearchableOutcome;
use dms_application::port::pdf::PageChange;
use dms_domain::catalog::{Catalog, CatalogUpdate, LibraryId};
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::RelativePath;

use super::*;
use crate::features::pdf::dto::PageItemDto;

fn document(index: u32, path: &str, title: Option<&str>) -> Document {
    let start: DateTime<Utc> = "2024-01-01T10:00:00Z".parse().expect("instant");
    let mut document = Document::new(
        DocumentId::new(format!("d{index}")).expect("id"),
        RelativePath::of(path).expect("path"),
        ContentHash::new(format!("{index:064x}")).expect("hash"),
        10,
        start + Duration::days(i64::from(index)),
    )
    .expect("document");
    document.metadata.title = title.map(str::to_owned);
    document
}

fn catalog(documents: Vec<Document>) -> Catalog {
    documents.into_iter().fold(Catalog::empty(LibraryId::new("lib").expect("id")), |catalog, document| {
        match catalog.add_document(document).expect("valid") {
            CatalogUpdate::Applied(next) => next,
            CatalogUpdate::Rejected(reason) => panic!("rejected: {reason:?}"),
        }
    })
}

fn page(source: u32) -> PageItemDto {
    PageItemDto { source, rotation: 0, deleted: false, split_after: false }
}

#[test]
fn merge_starts_with_the_document_and_a_merged_name() {
    let start = merge_start(&document(1, "Bank/statement.2024.pdf", None));

    assert_eq!(start.name, "statement.2024 (merged)");
    assert_eq!(start.first.name, "statement.2024.pdf");
}

#[test]
fn merge_matches_other_pdfs_by_path_or_title_newest_first() {
    let catalog = catalog(vec![
        document(1, "Bank/a.pdf", None),
        document(2, "Bank/b.pdf", Some("Loan contract")),
        document(3, "Home/contract.pdf", None),
        document(4, "Home/contract.png", None),
    ]);

    let all: Vec<String> = merge_matches(&catalog, "", &["d1".to_owned()]).into_iter().map(|m| m.document_id).collect();
    let found: Vec<String> = merge_matches(&catalog, " CONTRACT ", &[]).into_iter().map(|m| m.document_id).collect();

    assert_eq!(all, ["d3", "d2"]);
    assert_eq!(found, ["d3", "d2"]);
}

#[test]
fn untouched_pages_without_splits_change_nothing() {
    let edit = page_edit(&[page(0), page(1)]).expect("pages kept");

    assert_eq!(edit, PageEdit { changes: None, ranges: vec![0..=1] });
}

#[test]
fn turned_moved_and_removed_pages_form_the_new_version_and_split_on_kept_pages() {
    let pages = [
        PageItemDto { rotation: -90, split_after: true, ..page(2) },
        PageItemDto { deleted: true, ..page(0) },
        page(1),
        PageItemDto { split_after: true, ..page(3) },
    ];

    let edit = page_edit(&pages).expect("pages kept");

    assert_eq!(
        edit.changes,
        Some(vec![
            PageChange { page: 2, rotation: 270 },
            PageChange { page: 1, rotation: 0 },
            PageChange { page: 3, rotation: 0 }
        ])
    );
    assert_eq!(edit.ranges, vec![0..=0, 1..=2]);
}

#[test]
fn removing_every_page_is_refused() {
    assert_eq!(page_edit(&[PageItemDto { deleted: true, ..page(0) }]), None);
}

#[test]
fn searchable_outcomes_are_phrased_with_the_file_name() {
    let done = OperationOutcome::Done { catalog: catalog(Vec::new()), pending_folders: Default::default() };

    assert_eq!(
        searchable_message(&SearchableOutcome::Made(done), "scan.pdf"),
        UiMessage::info("scan.pdf is searchable now")
    );
    assert_eq!(
        searchable_message(&SearchableOutcome::NoTranscription, "scan.pdf"),
        UiMessage::error("scan.pdf has no transcription yet: classify it with AI first, or use precise OCR")
    );
}
