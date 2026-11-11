use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use dms_application::common::{NoProgress, Progress};
use dms_application::pdf::rebuild::{RebuildBlocker, RebuildOutcome};
use dms_application::port::ai::{ClassificationFailure, LayoutOutcome, LineOutcome};
use dms_domain::rebuild::{FontFamily, KeepReason, LayoutBox, LayoutLine, PageLayout};
use indexmap::IndexMap;

use crate::pdf::kit::PdfKit;
use crate::support::fake::pdf::FakePdfRebuilder;
use crate::support::fake::storage::FakeFolders;

fn read(lines: Vec<LayoutLine>) -> LayoutOutcome {
    LayoutOutcome::Read(PageLayout { lines, graphics: Vec::new() })
}

fn line(text: &str, top: f64) -> LayoutLine {
    LayoutLine {
        bounds: LayoutBox { left: 100.0, top, right: 600.0, bottom: top + 30.0 },
        text: text.to_owned(),
        bold: false,
        family: FontFamily::Sans,
        legible: true,
        kept: None,
    }
}

fn timeout() -> LayoutOutcome {
    LayoutOutcome::Failure { reason: ClassificationFailure::Timeout, detail: None }
}

#[test]
fn a_page_the_ai_cannot_read_in_time_stays_as_it_was_and_the_next_page_is_still_rebuilt() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("letters/scan.pdf", "p1|p2");
    library.layout_reader.layouts.update(|layouts| {
        layouts.insert(0, timeout());
        layouts.insert(1, read(vec![line("Sample line", 100.0)]));
    });
    let pages = Mutex::new(Vec::new());

    let outcome =
        library.rebuild_pdf().run(&scan.id, &|(page, _): &(usize, usize)| pages.lock().expect("lock").push(*page));

    let RebuildOutcome::Rebuilt { rebuilt, original_size } = outcome else {
        panic!("expected Rebuilt, got {outcome:?}")
    };
    assert_eq!(library.rebuilder.waiting.lock().get(&rebuilt.id).map(String::as_str), Some("page 0 kept|Sample line"));
    assert_eq!(rebuilt.pages[0].kept_reasons, IndexMap::from([(KeepReason::PageFailed, 1)]));
    // Page 0, page 1, then writing the file (page == pages).
    assert_eq!(pages.into_inner().expect("lock"), vec![0, 1, 2]);
    assert_eq!(original_size, scan.size);
    assert_eq!(library.folders.content(&library.primary, "letters/scan.pdf").as_deref(), Some("p1|p2"));
}

#[test]
fn lost_characters_are_read_again_and_a_reading_that_does_not_fit_leaves_the_line_an_image() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1");
    let fits = line("Gr\0\0e aus K\0ln", 100.0);
    let differs = line("Herr M\0ller", 200.0);
    library.layout_reader.lines.update(|lines| {
        lines.insert(FakePdfRebuilder::crop_text(&fits.bounds), LineOutcome::Read(" Grüße  aus Köln ".to_owned()));
        lines.insert(FakePdfRebuilder::crop_text(&differs.bounds), LineOutcome::Read("Herr Mueller".to_owned()));
    });
    library.layout_reader.layouts.update(|layouts| layouts.insert(0, read(vec![fits, differs])));

    let outcome = library.rebuild_pdf().run(&scan.id, &NoProgress);

    let RebuildOutcome::Rebuilt { rebuilt, .. } = outcome else { panic!("expected Rebuilt, got {outcome:?}") };
    assert_eq!(library.rebuilder.waiting.lock().get(&rebuilt.id).map(String::as_str), Some("Grüße aus Köln/[image]"));
    assert_eq!(rebuilt.pages[0].kept_reasons, IndexMap::from([(KeepReason::LostCharacters, 1)]));
}

#[test]
fn an_ai_that_cannot_read_images_stops_the_run_and_writes_nothing() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1|p2");
    let no_vision = LayoutOutcome::Failure { reason: ClassificationFailure::ModelCannotReadImages, detail: None };
    library.layout_reader.layouts.update(|layouts| layouts.insert(0, no_vision));

    let outcome = library.rebuild_pdf().run(&scan.id, &NoProgress);

    assert_eq!(outcome, RebuildOutcome::NotAvailable(RebuildBlocker::ModelCannotReadImages));
    assert!(library.rebuilder.waiting.lock().is_empty());
}

#[test]
fn a_pdf_changed_during_the_run_is_not_written_since_its_pages_would_come_from_two_versions() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1|p2");
    let store = library.store.clone();
    let id = scan.id.clone();
    library.layout_reader.set_before_layout(move |page| {
        if page == 1 {
            let _ = store.commit(|catalog| Ok(catalog.update_content(&id, FakeFolders::hash_of("p1"), 2)));
        }
    });

    let outcome = library.rebuild_pdf().run(&scan.id, &NoProgress);

    assert_eq!(outcome, RebuildOutcome::NotAvailable(RebuildBlocker::SourceChanged));
    assert!(library.rebuilder.waiting.lock().is_empty());
}

/// Cancelled once the flag is set, like a cancelled coroutine.
struct Cancellable(Arc<AtomicBool>);

impl Progress<(usize, usize)> for Cancellable {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[test]
fn cancelling_during_a_page_rethrows_and_leaves_no_rebuilt_file() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1|p2");
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = cancelled.clone();
    library.layout_reader.set_before_layout(move |page| {
        if page == 1 {
            flag.store(true, Ordering::SeqCst);
        }
    });

    let outcome = library.rebuild_pdf().run(&scan.id, &Cancellable(cancelled));

    // Kotlin rethrew the CancellationException; Rust answers Cancelled.
    assert_eq!(outcome, RebuildOutcome::Cancelled);
    assert!(library.rebuilder.waiting.lock().is_empty());
}

#[test]
fn only_pdfs_without_a_text_layer_can_be_rebuilt() {
    let library = PdfKit::new();
    library.open();
    let scan = library.add("scan.pdf", "p1");
    let searchable = library.add("text.pdf", "p1#text");
    let photo = library.add("photo.jpg", "jpg");

    assert!(library.rebuild_availability().run(&scan.id));
    assert!(!library.rebuild_availability().run(&searchable.id));
    assert!(!library.rebuild_availability().run(&photo.id));
    assert_eq!(
        library.rebuild_pdf().run(&searchable.id, &NoProgress),
        RebuildOutcome::NotAvailable(RebuildBlocker::HasTextLayer)
    );
}
