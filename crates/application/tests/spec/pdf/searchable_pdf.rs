use std::mem::discriminant;
use std::sync::Mutex;

use dms_application::pdf::{ScansOutcome, SearchableOutcome, SearchableProgress};
use dms_application::port::storage::TextStore;
use dms_domain::document::DocumentMetadata;
use dms_domain::settings::LibraryLayout;

use super::kit::PdfKit;
use crate::support::test_data::done;

#[test]
fn a_scan_gets_the_transcription_as_a_text_layer_one_text_per_page() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1|p2");
    library.folders.write(&library.primary, &scan.id, "first page\u{000C}second page").expect("written");

    let outcome = library.make_searchable().run(&scan.id, false);

    assert!(matches!(outcome, SearchableOutcome::Made(_)), "{outcome:?}");
    assert_eq!(library.folders.content(&library.sync_b, "scan.pdf").as_deref(), Some("p1#first page|p2#second page"));
}

#[test]
fn a_pdf_with_text_is_left_alone_and_one_without_transcription_says_so() {
    let library = PdfKit::new();
    library.open();
    let searchable = library.add("text.pdf", "p1#text");
    let scan = library.add("scan.pdf", "p1");
    let photo = library.add("photo.jpg", "jpg");

    assert_eq!(library.make_searchable().run(&searchable.id, false), SearchableOutcome::AlreadySearchable);
    assert_eq!(library.make_searchable().run(&scan.id, false), SearchableOutcome::NoTranscription);
    assert_eq!(library.make_searchable().run(&photo.id, false), SearchableOutcome::NotPdf);
    assert_eq!(library.folders.content(&library.primary, "text.pdf").as_deref(), Some("p1#text"));
}

#[test]
fn precise_ocr_needs_the_program_installed() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1");

    assert_eq!(library.make_searchable().run(&scan.id, true), SearchableOutcome::OcrUnavailable);
    library.ocr.installed.set(true);
    let outcome = library.make_searchable().run(&scan.id, true);
    assert!(matches!(outcome, SearchableOutcome::Made(_)), "{outcome:?}");
    assert_eq!(library.folders.content(&library.primary, "scan.pdf").as_deref(), Some("p1#ocr"));
}

#[test]
fn the_batch_skips_pdfs_that_have_text_and_finds_the_scans_first() {
    let library = PdfKit::new();
    library.open();
    library.add("a-text.pdf", "p1#text");
    let scan = library.add("b-scan.pdf", "p1");
    library.folders.write(&library.primary, &scan.id, "words").expect("written");

    let before = library.scans_without_text.run();
    let progress = Mutex::new(Vec::<SearchableProgress>::new());
    let _ = library
        .make_all_searchable()
        .run(false, &|step: &SearchableProgress| progress.lock().expect("lock").push(step.clone()));

    assert_eq!(before, ScansOutcome::Found(vec![scan.id.clone()]));
    let kinds: Vec<_> = progress.into_inner().expect("lock").iter().map(|step| discriminant(&step.outcome)).collect();
    let made = SearchableOutcome::Made(dms_application::common::OperationOutcome::NotFound);
    assert_eq!(kinds, vec![discriminant(&SearchableOutcome::AlreadySearchable), discriminant(&made)]);
    assert_eq!(library.scans_without_text.run(), ScansOutcome::Found(Vec::new()));
}

#[test]
fn metadata_is_only_embedded_while_the_librarys_setting_is_on() {
    let library = PdfKit::new();
    library.open();
    let bill = library.add("bill.pdf", "p1");
    done(library.store.commit(|catalog| {
        catalog.update_metadata(
            &bill.id,
            DocumentMetadata { title: Some("Power bill".to_owned()), ..DocumentMetadata::default() },
        )
    }));

    let _ = library.embed_pdf_metadata().run(&bill.id);
    assert_eq!(library.folders.content(&library.primary, "bill.pdf").as_deref(), Some("p1"));

    let _ = library.change_settings().run("embed", |settings| {
        settings.with_layout(LibraryLayout { embed_metadata: true, ..settings.layout.clone() })
    });
    let _ = library.embed_pdf_metadata().run(&bill.id);

    assert_eq!(library.folders.content(&library.sync_a, "bill.pdf").as_deref(), Some("p1;Power bill"));
}
