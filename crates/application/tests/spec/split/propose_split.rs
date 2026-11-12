use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::common::{NoProgress, Progress};
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::assistant::{SplitAnswer, SplitPage};
use dms_application::port::storage::TextStore;
use dms_application::split::{PageSource, ProposeSplit, ProposeSplitOutcome, SplitProgress};
use dms_domain::document::DocumentId;
use dms_domain::split::{SplitPart, SplitProblem, SplitProposal};

use super::kit::SplitKit;
use crate::support::fake::assistant::FakeSplitProposer;

fn proposed(outcome: ProposeSplitOutcome) -> (SplitProposal, PageSource) {
    match outcome {
        ProposeSplitOutcome::Proposed { proposal, source, .. } => (proposal, source),
        other => panic!("expected a proposal, got {other:?}"),
    }
}

fn answer(kit: &SplitKit, parts: Vec<SplitPart>) {
    kit.proposer.answer.set(SplitAnswer::Proposed { parts, model: FakeSplitProposer::MODEL.to_owned() });
}

fn titled(first: usize, last: usize, title: &str) -> SplitPart {
    SplitPart { title: Some(title.to_owned()), ..SplitPart::pages(first, last) }
}

/// Cancels once `pages` pages were reported.
struct CancelAfter {
    pages: usize,
    reported: AtomicUsize,
}

impl Progress<SplitProgress> for CancelAfter {
    fn report(&self, progress: &SplitProgress) {
        self.reported.store(progress.done, Ordering::SeqCst);
    }

    fn is_cancelled(&self) -> bool {
        self.reported.load(Ordering::SeqCst) >= self.pages
    }
}

#[test]
fn a_stack_with_a_text_layer_sends_its_page_texts_cut_short_with_blank_flags_and_keeps_valid_parts() {
    let kit = SplitKit::new();
    let long = "x".repeat(ProposeSplit::PAGE_TEXT_LIMIT + 50);
    let scan = kit.add("Inbox/stack.pdf", &format!("p#Invoice 1|blank#  |p#{long}"));
    answer(&kit, vec![titled(0, 1, "Invoice"), titled(2, 2, "Letter")]);

    let (proposal, source) = proposed(kit.propose_split().run(&scan.id, &NoProgress));

    assert_eq!(source, PageSource::TextLayer);
    assert_eq!(proposal, SplitProposal { parts: vec![titled(0, 1, "Invoice"), titled(2, 2, "Letter")], problem: None });
    let request = kit.proposer.requests.get().pop().expect("one request");
    assert_eq!(
        request.pages[..2],
        [SplitPage { text: "Invoice 1".to_owned(), blank: false }, SplitPage { text: String::new(), blank: true }]
    );
    assert_eq!(request.pages[2].text.chars().count(), ProposeSplit::PAGE_TEXT_LIMIT);
    assert_eq!(kit.proposer.pages_read.get(), Vec::<String>::new());
}

#[test]
fn parts_that_miss_pages_become_one_part_with_the_reason() {
    let kit = SplitKit::new();
    let scan = kit.add("stack.pdf", "a#one|b#two|c#three");
    answer(&kit, vec![titled(0, 0, "First")]);

    let (proposal, _) = proposed(kit.propose_split().run(&scan.id, &NoProgress));

    assert_eq!(proposal, SplitProposal { parts: vec![SplitPart::pages(0, 2)], problem: Some(SplitProblem::Gap) });
}

#[test]
fn a_scan_uses_the_stored_transcription_when_it_kept_one_text_per_page() {
    let kit = SplitKit::new();
    let scan = kit.add("scan.pdf", "a|b");
    kit.folders.write(&kit.primary, &scan.id, "Page one\u{c}Page two").expect("text stored");
    answer(&kit, vec![SplitPart::pages(0, 0), SplitPart::pages(1, 1)]);

    let (_, source) = proposed(kit.propose_split().run(&scan.id, &NoProgress));

    assert_eq!(source, PageSource::Transcription);
    let request = kit.proposer.requests.get().pop().expect("one request");
    assert_eq!(request.pages.iter().map(|page| page.text.as_str()).collect::<Vec<_>>(), ["Page one", "Page two"]);
}

#[test]
fn without_a_usable_transcription_the_vision_model_reads_every_page_that_is_not_blank() {
    let kit = SplitKit::new();
    let scan = kit.add("scan.pdf", "front|blank|back");
    kit.folders.write(&kit.primary, &scan.id, "Only the first two pages\u{c}were read").expect("text stored");
    let reported = std::sync::Mutex::new(Vec::new());
    let progress = |step: &SplitProgress| reported.lock().expect("lock").push(step.done);

    let (_, source) = proposed(kit.propose_split().run(&scan.id, &progress));

    assert_eq!(source, PageSource::Vision);
    assert_eq!(kit.proposer.pages_read.get(), ["front", "back"]);
    let request = kit.proposer.requests.get().pop().expect("one request");
    assert_eq!(
        request.pages.iter().map(|page| page.text.as_str()).collect::<Vec<_>>(),
        ["read: front", "", "read: back"]
    );
    assert_eq!(*reported.lock().expect("lock"), [0, 1, 2, 3]);
}

#[test]
fn cancelling_the_page_reading_stops_before_the_next_page_and_asks_for_no_proposal() {
    let kit = SplitKit::new();
    let scan = kit.add("scan.pdf", "a|b|c|d");

    let outcome = kit.propose_split().run(&scan.id, &CancelAfter { pages: 2, reported: AtomicUsize::new(0) });

    assert_eq!(outcome, ProposeSplitOutcome::Cancelled);
    assert_eq!(kit.proposer.pages_read.get(), ["a", "b"]);
    assert_eq!(kit.proposer.requests.get().len(), 0, "no proposal asked");
}

#[test]
fn a_page_the_model_cannot_read_and_a_failed_proposal_are_failures() {
    let kit = SplitKit::new();
    let scan = kit.add("scan.pdf", "a|b");
    kit.proposer.reading_fails.set(true);

    let unreadable = kit.propose_split().run(&scan.id, &NoProgress);

    assert_eq!(
        unreadable,
        ProposeSplitOutcome::Failed { reason: ClassificationFailure::ModelCannotReadImages, detail: None }
    );
    let text = kit.add("text.pdf", "a#one|b#two");
    kit.proposer.answer.set(SplitAnswer::Failed { reason: ClassificationFailure::Timeout, detail: None });
    let failed = kit.propose_split().run(&text.id, &NoProgress);
    assert_eq!(failed, ProposeSplitOutcome::Failed { reason: ClassificationFailure::Timeout, detail: None });
}

#[test]
fn single_pages_other_files_and_unknown_documents_are_refused() {
    let kit = SplitKit::new();
    let single = kit.add("single.pdf", "a#one");
    let photo = kit.add("photo.jpg", "jpeg");

    assert_eq!(kit.propose_split().run(&single.id, &NoProgress), ProposeSplitOutcome::TooFewPages);
    assert_eq!(kit.propose_split().run(&photo.id, &NoProgress), ProposeSplitOutcome::NotPdf);
    let unknown = DocumentId::new("unknown").expect("valid id");
    assert_eq!(kit.propose_split().run(&unknown, &NoProgress), ProposeSplitOutcome::NotFound);
}
