use dms_application::resolution::MoveDecision;
use dms_domain::change::{DetectedChange, MovedFile};

use super::*;
use crate::features::dashboard::dto::{DashboardAction, MoveChoice};
use crate::features::dashboard::plan::{Call, Summary, Work, plan, preferred_of};
use crate::session::Lane;

fn moved() -> MovedFile {
    MovedFile { document_id: id("d1"), path: path("b.pdf"), previous_path: path("a.pdf"), folder_id: folder_id(SYNC) }
}

#[test]
fn adopting_forgets_the_new_file_once_done() {
    let change = new_file("x.pdf", Some(9), &[PRIMARY]);
    let built = dashboard(vec![OpenTask::AdoptNewFile(change.clone())]);
    let plan = plan(&DashboardAction::Adopt { task_key: "new:x.pdf".to_owned() }, &built, Library::new().catalog())
        .expect("a plan");
    assert_eq!(plan.label, "Adopting x.pdf");
    assert_eq!(plan.lane, Lane::Library);
    assert_eq!(
        plan.work,
        Work::One {
            call: Call::AdoptNewFile(change.clone()),
            done: Some("Adopted x.pdf".to_owned()),
            resolves: Some(DetectedChange::NewFile(change)),
        }
    );
}

#[test]
fn resolving_a_move_names_both_directions() {
    let built = dashboard(vec![OpenTask::ConfirmMove(moved())]);
    let key = "moved:d1:f-sync".to_owned();
    let apply = plan(
        &DashboardAction::ResolveMove { task_key: key.clone(), choice: MoveChoice::Apply },
        &built,
        Library::new().catalog(),
    )
    .expect("a plan");
    let Work::One { call, done, .. } = apply.work else { panic!("one call") };
    assert_eq!(call, Call::ResolveMove { change: moved(), decision: MoveDecision::Apply });
    assert_eq!(done.as_deref(), Some("Moved a.pdf to b.pdf everywhere"));
    let undo = plan(
        &DashboardAction::ResolveMove { task_key: key, choice: MoveChoice::Undo },
        &built,
        Library::new().catalog(),
    )
    .expect("a plan");
    let Work::One { done, .. } = undo.work else { panic!("one call") };
    assert_eq!(done.as_deref(), Some("Moved b.pdf back to a.pdf"));
}

#[test]
fn keeping_a_version_is_labelled_with_the_folder_name() {
    let change = dms_domain::change::ModifiedFile {
        document_id: id("d1"),
        path: path("a.pdf"),
        folder_id: folder_id(SYNC),
        size: 1,
        content_hash: None,
    };
    let built = dashboard(vec![OpenTask::ReviewModification(change)]);
    let action = DashboardAction::KeepVersion { task_key: "modified:d1:f-sync".to_owned(), folder_id: SYNC.to_owned() };
    let plan = plan(&action, &built, Library::new().catalog()).expect("a plan");
    assert_eq!(plan.label, "Keeping the version of Cloud");
    let Work::One { call, done, .. } = plan.work else { panic!("one call") };
    assert_eq!(call, Call::KeepVersion { path: path("a.pdf"), source: folder_id(SYNC) });
    assert_eq!(done.as_deref(), Some("Kept the version of Cloud"));
}

#[test]
fn a_task_that_is_gone_or_an_action_of_the_window_is_refused() {
    let built = dashboard(Vec::new());
    let error = plan(&DashboardAction::Adopt { task_key: "new:x.pdf".to_owned() }, &built, Library::new().catalog())
        .expect_err("refused");
    assert_eq!(error.code, "invalid_input");
    assert!(plan(&DashboardAction::OpenInbox, &built, Library::new().catalog()).is_err());
}

#[test]
fn bulk_runs_are_cancellable_and_report_counts() {
    let library = Library::new().document_with("bare", "a.pdf", "2025-01-01T00:00:00Z", 3, nothing).document_with(
        "worked",
        "b.pdf",
        "2025-02-01T00:00:00Z",
        3,
        tagged(&["car"]),
    );
    let built = dashboard(vec![
        OpenTask::ConfirmMove(moved()),
        OpenTask::AdoptNewFile(new_file("copy.pdf", Some(3), &[PRIMARY])),
        OpenTask::ResolveDuplicates { content_hash: hash(3), document_ids: vec![id("bare"), id("worked")] },
    ]);
    let rename = plan(&DashboardAction::RenameAll, &built, library.catalog()).expect("a plan");
    assert_eq!((rename.label.as_str(), rename.cancellable), ("Renaming 1 files to match", true));
    let remove = plan(&DashboardAction::RemoveAllDuplicates, &built, library.catalog()).expect("a plan");
    let Work::Each { phases, summary } = remove.work else { panic!("a run over many") };
    assert_eq!(summary, Summary::Files("Removed duplicates of"));
    assert_eq!(phases[0].0, "Removing copy");
    assert_eq!(phases[0].1[0].name, "copy.pdf");
    assert_eq!(phases[1].1[0].call, Call::MergePreferredOf(vec![id("bare"), id("worked")]));
    assert_eq!(preferred_of(&[id("bare"), id("worked")], library.catalog()), Some(id("worked")));
    assert_eq!(preferred_of(&[id("bare"), id("gone")], library.catalog()), None);
}

#[test]
fn trashing_retained_documents_counts_the_known_ones() {
    let library = Library::new().document("d1", "a.pdf", "2025-01-01T00:00:00Z", nothing);
    let action = DashboardAction::TrashRetained { document_ids: vec!["d1".to_owned(), "gone".to_owned()] };
    let plan = plan(&action, &dashboard(Vec::new()), library.catalog()).expect("a plan");
    assert_eq!(plan.label, "Moving 1 documents to the trash");
    let Work::Each { phases, summary } = plan.work else { panic!("a run over many") };
    assert_eq!(summary, Summary::TrashedDocuments);
    assert_eq!(phases[0].1.len(), 2);
}
