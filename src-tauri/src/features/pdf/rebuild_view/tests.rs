#![allow(clippy::default_trait_access)] // the IndexMap fields are built without naming indexmap, not a dependency here

use dms_application::common::OperationOutcome;
use dms_application::pdf::rebuild::{RebuildBlocker, RebuildChoice, RebuildOutcome};
use dms_application::port::pdf::RebuiltFile;
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::RelativePath;
use dms_domain::rebuild::{KeepReason, LayoutBox, RebuildRegion, RebuiltPage, RegionKind};

use super::*;

fn page(kept: &[(KeepReason, usize)]) -> RebuiltPage {
    let mut page = RebuiltPage {
        lines_as_text: 8,
        lines_total: 10,
        angle: 0.0,
        kept_reasons: Default::default(),
        regions: vec![RebuildRegion {
            bounds: LayoutBox { left: 10.0, top: 20.0, right: 500.0, bottom: 40.0 },
            kind: RegionKind::KeptLine,
        }],
    };
    for (reason, count) in kept {
        page.kept_reasons.insert(*reason, *count);
    }
    page
}

fn rebuilt(pages: Vec<RebuiltPage>) -> RebuiltFile {
    RebuiltFile { id: "r1".to_owned(), pages, size: 2048, source_hash: ContentHash::new("0".repeat(64)).expect("hash") }
}

#[test]
fn a_rebuild_with_unreadable_pages_names_them() {
    let one = RebuildOutcome::Rebuilt {
        rebuilt: rebuilt(vec![page(&[]), page(&[(KeepReason::PageFailed, 1)])]),
        original_size: 1,
    };
    let two = RebuildOutcome::Rebuilt {
        rebuilt: rebuilt(vec![page(&[(KeepReason::PageFailed, 1)]), page(&[]), page(&[(KeepReason::PageFailed, 1)])]),
        original_size: 1,
    };
    let fine = RebuildOutcome::Rebuilt { rebuilt: rebuilt(vec![page(&[])]), original_size: 1 };

    assert_eq!(
        rebuild_message(&one, "scan.pdf"),
        Some(UiMessage::info("Page 2 stayed an image (the AI could not read it in time or at all)"))
    );
    assert_eq!(
        rebuild_message(&two, "scan.pdf"),
        Some(UiMessage::info("Pages 1, 3 stayed images (the AI could not read it in time or at all)"))
    );
    assert_eq!(rebuild_message(&fine, "scan.pdf"), Some(UiMessage::info("scan.pdf is rebuilt: compare and choose")));
    assert_eq!(rebuild_message(&RebuildOutcome::Cancelled, "scan.pdf"), None);
    assert_eq!(
        rebuild_message(&RebuildOutcome::NotAvailable(RebuildBlocker::HasTextLayer), "scan.pdf"),
        Some(UiMessage::error("scan.pdf has real text already"))
    );
}

#[test]
fn progress_counts_pages_then_writes() {
    assert_eq!(progress_of(0, 3).step, "Page 1 of 3 · a few minutes per page");
    assert_eq!(progress_of(3, 3).step, "Writing the PDF");
}

#[test]
fn the_comparison_lists_kept_reasons_most_frequent_first_and_the_regions() {
    let comparison = Comparison {
        document_id: DocumentId::new("d1").expect("id"),
        path: RelativePath::of("Scans/scan.pdf").expect("path"),
        rebuilt: rebuilt(vec![page(&[(KeepReason::NoInk, 1), (KeepReason::NotSure, 3)])]),
        original_size: 4096,
    };

    let dto = present_compare(&comparison);

    assert_eq!(dto.name, "scan.pdf");
    assert_eq!(
        dto.pages[0].kept.as_deref(),
        Some("Kept as images: 3 × the AI was not sure of every character; 1 × no print where the AI saw the line")
    );
    assert_eq!(dto.pages[0].regions[0].kind, RegionKindDto::KeptLine);
    assert!((dto.original_size - 4096.0).abs() < f64::EPSILON);
}

#[test]
fn a_changed_scan_keeps_the_comparison_open_with_its_own_message() {
    let outcome = OperationOutcome::SourceChanged;

    assert_eq!(
        choice_message(&outcome, RebuildChoice::ReplaceWithRebuild, "scan.pdf"),
        UiMessage::error("scan.pdf changed after it was converted: convert it again, or keep both")
    );
    assert!(!closes_comparison(&outcome));
    assert!(closes_comparison(&OperationOutcome::NotFound));
}
