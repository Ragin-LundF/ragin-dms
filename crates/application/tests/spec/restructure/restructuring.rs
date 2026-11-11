use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};

use dms_application::common::{NoProgress, OperationOutcome, Progress};
use dms_application::port::storage::FolderOperation;
use dms_application::restructure::{PlanOutcome, RestructureOutcome, RestructureScope};
use dms_domain::catalog::CatalogUpdate;
use dms_domain::document::{Document, DocumentMetadata};
use dms_domain::library::RelativePath;
use dms_domain::restructure::{FileNaming, PlannedMove, RestructurePlan, TemplateProblem};
use dms_domain::taxonomy::Category;
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::test_data::{category_id, date, done, path};

fn bills() -> Category {
    Category::new(category_id("bills"), "Bills", None, None).expect("valid category")
}

fn library_with(count: u32) -> (TestLibrary, Vec<Document>) {
    let library = TestLibrary::new();
    library.open();
    done(library.store.commit(|catalog| catalog.add_category(bills())));
    let documents = (1..=count)
        .map(|index| {
            let document = library.add(&format!("loose/bill-{index}.pdf"), &format!("bill {index}"));
            let metadata =
                DocumentMetadata { document_date: Some(date(2024, 1, index % 28 + 1)), ..DocumentMetadata::default() };
            done(library.store.commit(|catalog| match catalog.assign_category(&document.id, Some(bills().id)) {
                CatalogUpdate::Applied(assigned) => assigned.update_metadata(&document.id, metadata.clone()),
                rejected @ CatalogUpdate::Rejected(_) => Ok(rejected),
            }));
            document
        })
        .collect();
    (library, documents)
}

fn plan(library: &TestLibrary) -> RestructurePlan {
    let outcome =
        library.plan_restructure().run(&RestructureScope::Folder(path("loose")), "{category}/{year}", FileNaming::Keep);
    let PlanOutcome::Planned(plan) = outcome else { panic!("expected Planned, got {outcome:?}") };
    plan
}

struct Applied {
    moved: usize,
    failed: Vec<PlannedMove>,
    not_catalogued: Vec<PlannedMove>,
    skipped: usize,
    pending_folders: IndexSet<dms_domain::library::LibraryFolderId>,
    record: Option<String>,
    cancelled: bool,
}

#[track_caller]
fn applied(outcome: RestructureOutcome) -> Applied {
    match outcome {
        RestructureOutcome::Applied {
            moved,
            failed,
            not_catalogued,
            skipped,
            pending_folders,
            record,
            cancelled,
            ..
        } => Applied { moved, failed, not_catalogued, skipped, pending_folders, record, cancelled },
        other => panic!("expected Applied, got {other:?}"),
    }
}

fn apply(library: &TestLibrary, plan: &RestructurePlan, progress: &dyn Progress<(usize, usize)>) -> Applied {
    applied(library.apply_restructure_plan().run(plan, 0, progress))
}

fn strings(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|path| (*path).to_owned()).collect()
}

fn catalog_paths(library: &TestLibrary) -> Vec<String> {
    library.catalog().documents().values().map(|document| document.path.value().to_owned()).collect()
}

/// Cancels once `(done, _)` reached `at`, as the Kotlin test threw a `CancellationException` from `onProgress`.
struct CancelAt {
    at: usize,
    cancelled: AtomicBool,
}

impl Progress<(usize, usize)> for CancelAt {
    fn report(&self, &(done, _): &(usize, usize)) {
        if done == self.at {
            self.cancelled.store(true, Ordering::SeqCst);
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[test]
fn an_invalid_template_is_refused_before_anything_is_planned() {
    let (library, _) = library_with(1);

    let outcome =
        library.plan_restructure().run(&RestructureScope::Folder(RelativePath::root()), "{nonsense}", FileNaming::Keep);

    assert_eq!(outcome, PlanOutcome::InvalidTemplate(TemplateProblem::UnknownPlaceholder));
}

#[test]
fn applying_moves_every_file_in_every_folder_and_records_the_plan() {
    let (library, documents) = library_with(3);

    let outcome = apply(&library, &plan(&library), &NoProgress);

    assert_eq!(outcome.moved, 3);
    let record = outcome.record.expect("a record");
    for folder in library.all_folders() {
        assert_eq!(
            library.folders.paths(&folder),
            strings(&["Bills/2024/bill-1.pdf", "Bills/2024/bill-2.pdf", "Bills/2024/bill-3.pdf"])
        );
        assert!(library.plans.records_in(&folder).contains_key(&record));
    }
    assert_eq!(library.catalog().documents()[&documents[0].id].path.value(), "Bills/2024/bill-1.pdf");
}

#[test]
fn the_catalog_is_committed_in_batches_of_fifty() {
    let (library, _) = library_with(120);
    let before = library.catalog().revision();

    apply(&library, &plan(&library), &NoProgress);

    // Three batch commits, plus none for the plan file, which is no catalog change.
    assert_eq!(library.catalog().revision().value(), before.value() + 3);
}

#[test]
fn an_offline_sync_folder_gets_its_moves_from_the_journal() {
    let (library, _) = library_with(2);
    library.folders.make_unreachable(&library.sync_b);

    let outcome = apply(&library, &plan(&library), &NoProgress);

    assert_eq!(outcome.pending_folders, IndexSet::from([library.sync_b.id.clone()]));
    let pending = library.journal.operations.get();
    assert_eq!(pending.iter().filter(|pending| matches!(pending.operation, FolderOperation::Move { .. })).count(), 2);
    assert!(pending.iter().any(|pending| matches!(pending.operation, FolderOperation::CopyPlan(_))));
    assert_eq!(library.folders.paths(&library.sync_b), strings(&["loose/bill-1.pdf", "loose/bill-2.pdf"]));
}

#[test]
fn cancelling_midway_commits_what_was_moved_so_the_catalog_matches_the_disk() {
    let (library, _) = library_with(4);
    let plan = plan(&library);

    let outcome = apply(&library, &plan, &CancelAt { at: 2, cancelled: AtomicBool::new(false) });

    assert!(outcome.cancelled);
    let moved = catalog_paths(&library).iter().filter(|path| path.starts_with("Bills/")).count();
    assert_eq!(moved, 2);
    for document in library.catalog().documents().values() {
        assert!(library.folders.content(&library.primary, document.path.value()).is_some());
    }
    assert_eq!(library.plans.records_in(&library.primary).len(), 1);
}

#[test]
fn reversing_puts_documents_back_except_one_the_user_moved_since() {
    let (library, documents) = library_with(3);
    let applied = apply(&library, &plan(&library), &NoProgress);
    done(library.move_document().run(&documents[0].id, &path("kept/bill-1.pdf")));

    let reversed = self::applied(library.reverse_restructure().run(&applied.record.expect("a record"), &NoProgress));

    assert_eq!(reversed.moved, 2);
    assert_eq!(reversed.skipped, 1);
    assert_eq!(
        catalog_paths(&library).into_iter().collect::<BTreeSet<_>>(),
        strings(&["kept/bill-1.pdf", "loose/bill-2.pdf", "loose/bill-3.pdf"])
    );
}

#[test]
fn moves_whose_catalog_commit_failed_are_reported_apart_recorded_and_reversed() {
    let (library, _) = library_with(60);
    let folders = library.folders.clone();
    let primary = library.primary.clone();
    let read_only_at_51 = move |&(done, _): &(usize, usize)| {
        if done == 51 {
            folders.make_catalog_read_only(&primary);
        }
    };

    let applied = apply(&library, &plan(&library), &read_only_at_51);
    library.folders.state.update(|state| state.read_only_catalogs.clear());

    assert_eq!(applied.moved, 50);
    assert_eq!(applied.not_catalogued.len(), 10);
    assert_eq!(applied.failed, Vec::<PlannedMove>::new());
    let record = applied.record.expect("a record");
    let recorded = library.plans.records_in(&library.primary).get(&record).cloned().expect("the record");
    assert_eq!(recorded.moves.len(), 60);
    for planned in &applied.not_catalogued {
        assert_eq!(library.catalog().documents()[&planned.document_id].path, planned.from);
        assert!(library.folders.content(&library.primary, planned.to.value()).is_some());
    }

    let reversed = self::applied(library.reverse_restructure().run(&record, &NoProgress));

    assert_eq!(reversed.moved, 60);
    assert!(catalog_paths(&library).iter().all(|path| path.starts_with("loose/")));
    assert!(library.folders.paths(&library.primary).iter().all(|path| path.starts_with("loose/")));
}

#[test]
fn a_batch_whose_catalog_commit_failed_once_is_committed_at_the_end() {
    let (library, _) = library_with(120);
    let folders = library.folders.clone();
    let primary = library.primary.clone();
    let toggle = move |&(done, _): &(usize, usize)| match done {
        51 => folders.make_catalog_read_only(&primary),
        100 => folders.state.update(|state| state.read_only_catalogs.clear()),
        _ => {}
    };

    let applied = apply(&library, &plan(&library), &toggle);

    assert_eq!(applied.moved, 120);
    assert_eq!(applied.not_catalogued, Vec::<PlannedMove>::new());
    assert!(catalog_paths(&library).iter().all(|path| path.starts_with("Bills/")));
}

#[test]
fn a_move_whose_document_changed_in_the_meantime_fails_and_the_rest_goes_on() {
    let (library, documents) = library_with(2);
    let plan = plan(&library);
    done(library.move_document().run(&documents[0].id, &path("elsewhere.pdf")));

    let outcome = apply(&library, &plan, &NoProgress);

    assert_eq!(outcome.moved, 1);
    assert_eq!(
        outcome.failed.iter().map(|planned| planned.document_id.clone()).collect::<Vec<_>>(),
        vec![documents[0].id.clone()]
    );
}

#[test]
fn empty_folders_are_only_removed_with_confirmation() {
    let (library, _) = library_with(1);
    let _ = library.create_folder().run(&path("old"));

    assert_eq!(library.remove_empty_folders().run(&[path("old")], false), OperationOutcome::NotConfirmed);
    done(library.remove_empty_folders().run(&[path("old"), path("loose")], true));
    for folder in library.all_folders() {
        assert!(!library.folders.has_directory(&folder, "old"));
        assert!(library.folders.paths(&folder).contains("loose/bill-1.pdf"));
    }
}

#[test]
fn the_history_lists_the_records_newest_first() {
    let (library, _) = library_with(1);
    apply(&library, &plan(&library), &NoProgress);

    let outcome = library.list_restructure_history().run();

    assert_eq!(
        outcome,
        dms_application::restructure::RestructureHistoryOutcome::Listed(vec!["20240301-090000".to_owned()])
    );
}
