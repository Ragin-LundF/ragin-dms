use std::collections::HashSet;

use dms_application::common::{NoProgress, OperationOutcome};
use dms_application::pdf::rebuild::{RebuildChoice, RebuildOutcome};
use dms_application::port::ai::LayoutOutcome;
use dms_application::port::pdf::RebuiltFile;
use dms_application::port::state::{ActivityKind, ActivityLog};
use dms_domain::document::Document;
use dms_domain::rebuild::{FontFamily, LayoutBox, LayoutLine, PageLayout};
use indexmap::IndexSet;

use crate::pdf::kit::PdfKit;
use crate::support::fake::storage::FakeFolders;
use crate::support::test_data::{done, path, tag};

const SCAN: &str = "letters/scan.pdf";

/// A tagged scan with one rebuilt page that reads "Sample line".
fn rebuilt_scan(library: &PdfKit) -> (Document, RebuiltFile) {
    library.open();
    let added = library.add(SCAN, "p1");
    done(library.store.commit(|catalog| Ok(catalog.set_tags(&added.id, &IndexSet::from([tag("Tax")])))));
    let line = LayoutLine {
        bounds: LayoutBox { left: 100.0, top: 100.0, right: 600.0, bottom: 130.0 },
        text: "Sample line".to_owned(),
        bold: false,
        family: FontFamily::Sans,
        legible: true,
        kept: None,
    };
    library.layout_reader.layouts.update(|layouts| {
        layouts.insert(0, LayoutOutcome::Read(PageLayout { lines: vec![line], graphics: Vec::new() }))
    });
    let outcome = library.rebuild_pdf().run(&added.id, &NoProgress);
    let RebuildOutcome::Rebuilt { rebuilt, .. } = outcome else { panic!("expected Rebuilt, got {outcome:?}") };
    (library.catalog().documents()[&added.id].clone(), rebuilt)
}

fn waiting_ids(library: &PdfKit) -> HashSet<String> {
    library.rebuilder.waiting.lock().keys().cloned().collect()
}

#[test]
fn keeping_the_original_changes_nothing_and_drops_the_rebuild() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);
    let before = library.catalog();

    let outcome = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::KeepOriginal);

    done(outcome);
    assert_eq!(library.catalog(), before);
    assert_eq!(library.folders.content(&library.primary, SCAN).as_deref(), Some("p1"));
    assert!(library.rebuilder.waiting.lock().is_empty());
}

#[test]
fn using_the_rebuild_replaces_the_file_in_every_folder_trashes_the_scan_and_logs_it() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);

    let outcome = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::ReplaceWithRebuild);

    done(outcome);
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, SCAN).as_deref(), Some("Sample line"));
    }
    let document = library.catalog().documents()[&scan.id].clone();
    assert_eq!(document.size, "Sample line".len() as u64);
    assert!(library.folders.trashed().contains(&(library.primary.id.clone(), scan.path.clone())));
    let entries = library.activity_log.recent(1).expect("log");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, ActivityKind::PdfEdited);
    assert_eq!(entries[0].subject, "Rebuilt: letters/scan.pdf");
    assert!(library.rebuilder.waiting.lock().is_empty());
}

#[test]
fn keeping_both_adds_the_rebuild_next_to_the_original_with_its_category_tags_and_fields() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);

    let outcome = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::KeepBoth);

    done(outcome);
    let copy = library.catalog().find_by_path(&path("letters/scan (rebuilt).pdf")).cloned().expect("the copy");
    assert_eq!(copy.tags, IndexSet::from([tag("Tax")]));
    assert_eq!(copy.fields, scan.fields);
    assert_ne!(copy.id, scan.id);
    for folder in library.all_folders() {
        assert_eq!(library.folders.content(&folder, SCAN).as_deref(), Some("p1"));
        assert_eq!(library.folders.content(&folder, "letters/scan (rebuilt).pdf").as_deref(), Some("Sample line"));
    }
    assert!(library.rebuilder.waiting.lock().is_empty());
}

#[test]
fn a_rebuild_that_cannot_be_placed_stays_for_another_choice() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);
    let before = library.catalog();
    library.folders.make_unreachable(&library.primary);

    let outcome = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::KeepBoth);

    assert!(matches!(outcome, OperationOutcome::StorageFailed(_)), "{outcome:?}");
    assert_eq!(library.catalog(), before);
    assert_eq!(waiting_ids(&library), HashSet::from([rebuilt.id.clone()]));
}

#[test]
fn a_scan_that_changed_after_the_conversion_is_not_replaced_and_keeping_both_still_works() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);
    library.folders.put(&library.primary, SCAN, "p1@90");
    done(library.store.commit(|catalog| Ok(catalog.update_content(&scan.id, FakeFolders::hash_of("p1@90"), 5))));

    let replaced = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::ReplaceWithRebuild);

    assert_eq!(replaced, OperationOutcome::SourceChanged);
    assert_eq!(library.folders.content(&library.primary, SCAN).as_deref(), Some("p1@90"));
    assert_eq!(waiting_ids(&library), HashSet::from([rebuilt.id.clone()]));
    done(library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::KeepBoth));
}

#[test]
fn a_rebuild_whose_document_is_gone_is_dropped_whatever_the_choice() {
    let library = PdfKit::new();
    let (scan, rebuilt) = rebuilt_scan(&library);
    done(library.store.commit(|catalog| Ok(catalog.remove_document(&scan.id))));

    let outcome = library.keep_rebuild().run(&scan.id, &rebuilt, RebuildChoice::KeepBoth);

    assert_eq!(outcome, OperationOutcome::NotFound);
    assert!(library.rebuilder.waiting.lock().is_empty());
}
