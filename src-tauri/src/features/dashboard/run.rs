//! Runs an [`ActionPlan`] as one operation of the session: calls its use cases, forgets the changes they resolved
//! and reports the outcome (Kotlin `DashboardViewModel.act`, `BulkActions`, `OrganizeTasks`).

use std::sync::Arc;

use dms_application::common::OperationOutcome;
use dms_application::port::storage::StorageResult;
use dms_application::services::AppServices;
use dms_domain::change::DetectedChange;

use super::plan::{ActionPlan, Call, Summary, Work, preferred_of};
use crate::dto::IpcError;
use crate::messages;
use crate::session::reporting::busy;
use crate::session::{Session, UiMessage};

/// Starts `plan` in its lane. `outcome_of` calls the use case of a [`Call`]; `forget` removes a resolved change
/// from the last scan.
///
/// # Errors
/// `busy` while another operation runs in the plan's lane.
pub fn execute<O, F>(session: &Arc<Session>, plan: ActionPlan, outcome_of: O, forget: F) -> Result<(), IpcError>
where
    O: Fn(&Call) -> OperationOutcome + Send + 'static,
    F: Fn(&DetectedChange) + Send + 'static,
{
    let (label, lane) = (plan.label.clone(), plan.lane);
    let started = session.perform(&plan.label, plan.lane, plan.cancellable, move |operation| {
        let resolve = |call: &Call, resolves: Option<&DetectedChange>| {
            let outcome = outcome_of(call);
            let done = matches!(outcome, OperationOutcome::Done { .. });
            if done && let Some(change) = resolves {
                forget(change);
            }
            outcome
        };
        match plan.work {
            Work::One { call, done, resolves } => {
                let outcome = resolve(&call, resolves.as_ref());
                if let Some(done) = done {
                    operation.session().report(messages::of(&outcome, &done));
                }
            }
            Work::Each { phases, summary } => {
                let total: usize = phases.iter().map(|(_, items)| items.len()).sum();
                let succeeded: usize = phases
                    .iter()
                    .map(|(step, items)| {
                        operation.for_each_with_progress(
                            step,
                            items,
                            |item| item.name.clone(),
                            |item| matches!(resolve(&item.call, item.resolves.as_ref()), OperationOutcome::Done { .. }),
                        )
                    })
                    .sum();
                operation.session().report(summary_message(summary, succeeded, total));
            }
        }
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(session, lane, &label)) }
}

/// Calls the use case `call` names.
pub fn outcome_of(services: &AppServices, call: &Call) -> OperationOutcome {
    let documents = &services.documents;
    let sync = &services.sync;
    match call {
        Call::AdoptNewFile(change) => documents.adopt_new_file.run(change),
        Call::RestoreDocument(change) => documents.restore_document.run(change),
        Call::TrashNewFile(change) => documents.trash_new_file.run(change, true),
        Call::TrashDuplicateFile(change) => documents.trash_duplicate_file.run(change, true),
        Call::TrashDocument(id) => documents.trash_document.run(id, true),
        Call::KeepVersion { path, source } => sync.keep_version.run(path, source, true),
        Call::KeepBothVersions { path, folder } => sync.keep_both_versions.run(path, folder),
        Call::ResolveMove { change, decision } => sync.resolve_move.run(change, *decision),
        Call::MergeDuplicates(keep) => sync.merge_duplicates.run(keep, true),
        Call::MergePreferredOf(ids) => {
            match services.library.store.catalog().and_then(|catalog| preferred_of(ids, &catalog)) {
                Some(keep) => sync.merge_duplicates.run(&keep, true),
                None => OperationOutcome::NotFound,
            }
        }
        Call::RejectSuggestion(id) => stored(services, services.ai.reject_suggestion.run(id)),
        Call::DismissTask(key) => stored(services, sync.dismiss_task.run(key)),
        Call::SettleRetention(ids) => documents.settle_retention.run(ids),
        Call::AcceptRuleProposal(proposal) => services.organize.accept_rule_proposal.run(proposal),
    }
}

/// Use cases that only write device state answer a `StorageResult`; as an outcome it is done with the open catalog.
fn stored(services: &AppServices, result: StorageResult<()>) -> OperationOutcome {
    match (result, services.library.store.catalog()) {
        (Err(error), _) => OperationOutcome::StorageFailed(error),
        (Ok(()), Some(catalog)) => OperationOutcome::Done { catalog, pending_folders: std::iter::empty().collect() },
        (Ok(()), None) => OperationOutcome::LibraryNotOpen,
    }
}

fn summary_message(summary: Summary, done: usize, total: usize) -> UiMessage {
    match summary {
        Summary::Files(verb) => {
            let text = format!("{verb} {done} of {total} files");
            if done < total { UiMessage::error(text) } else { UiMessage::info(text) }
        }
        Summary::TrashedDocuments => UiMessage::info(format!("Moved {done} of {total} documents to the trash")),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    use dms_domain::catalog::{Catalog, LibraryId};
    use dms_domain::change::NewFile;
    use dms_domain::document::DocumentId;
    use dms_domain::library::{LibraryFolderId, RelativePath};

    use super::*;
    use crate::session::{Lane, LogEntry, OperationProgress, SessionEvents, SessionState};

    struct Recorder {
        messages: Mutex<Vec<UiMessage>>,
        idle: Mutex<Sender<()>>,
    }

    impl SessionEvents for Recorder {
        fn state(&self, state: &SessionState) {
            if state.operations.is_empty() {
                let _ = self.idle.lock().expect("lock").send(());
            }
        }
        fn progress(&self, _: &HashMap<Lane, OperationProgress>) {}
        fn log(&self, _: &LogEntry) {}
        fn message(&self, message: &UiMessage) {
            self.messages.lock().expect("lock").push(message.clone());
        }
    }

    fn session() -> (Arc<Session>, Arc<Recorder>, Receiver<()>) {
        let (idle, finished) = channel();
        let recorder = Arc::new(Recorder { messages: Mutex::new(Vec::new()), idle: Mutex::new(idle) });
        (Session::new(recorder.clone()), recorder, finished)
    }

    fn done() -> OperationOutcome {
        OperationOutcome::Done {
            catalog: Catalog::empty(LibraryId::new("library").expect("id")),
            pending_folders: std::iter::empty().collect(),
        }
    }

    fn new_file(path: &str) -> NewFile {
        NewFile {
            path: RelativePath::of(path).expect("path"),
            present_in: std::iter::once(LibraryFolderId::new("primary").expect("id")).collect(),
            size: 10,
            content_hash: None,
        }
    }

    fn messages_after(recorder: &Recorder, finished: &Receiver<()>) -> Vec<UiMessage> {
        finished.recv_timeout(Duration::from_secs(5)).expect("the operation finishes");
        recorder.messages.lock().expect("lock").clone()
    }

    #[test]
    fn a_done_call_forgets_its_change_and_reports_the_done_text() {
        let (session, recorder, finished) = session();
        let forgotten = Arc::new(Mutex::new(Vec::new()));
        let change = DetectedChange::NewFile(new_file("a.pdf"));
        let plan = ActionPlan {
            label: "Adopting a.pdf".to_owned(),
            lane: Lane::Library,
            cancellable: false,
            work: Work::One {
                call: Call::AdoptNewFile(new_file("a.pdf")),
                done: Some("Adopted a.pdf".to_owned()),
                resolves: Some(change.clone()),
            },
        };
        let sink = Arc::clone(&forgotten);
        execute(&session, plan, |_| done(), move |change| sink.lock().expect("lock").push(change.clone()))
            .expect("started");
        assert_eq!(messages_after(&recorder, &finished), vec![UiMessage::info("Adopted a.pdf")]);
        assert_eq!(*forgotten.lock().expect("lock"), vec![change]);
    }

    #[test]
    fn a_refused_call_keeps_its_change_and_reports_why() {
        let (session, recorder, finished) = session();
        let forgotten = Arc::new(Mutex::new(0));
        let plan = ActionPlan {
            label: "Restoring".to_owned(),
            lane: Lane::Library,
            cancellable: false,
            work: Work::One {
                call: Call::TrashDocument(DocumentId::new("d1").expect("id")),
                done: Some("Restored".to_owned()),
                resolves: Some(DetectedChange::NewFile(new_file("a.pdf"))),
            },
        };
        let sink = Arc::clone(&forgotten);
        execute(&session, plan, |_| OperationOutcome::ScanOutdated, move |_| *sink.lock().expect("lock") += 1)
            .expect("started");
        assert_eq!(
            messages_after(&recorder, &finished),
            vec![UiMessage::error("The folders changed since the last scan; scan again")]
        );
        assert_eq!(*forgotten.lock().expect("lock"), 0);
    }

    #[test]
    fn a_bulk_run_counts_every_phase_and_is_an_error_when_one_failed() {
        let (session, recorder, finished) = session();
        let item = |path: &str| super::super::plan::Item {
            name: path.to_owned(),
            call: Call::TrashDuplicateFile(new_file(path)),
            resolves: None,
        };
        let plan = ActionPlan {
            label: "Removing duplicates".to_owned(),
            lane: Lane::Library,
            cancellable: true,
            work: Work::Each {
                phases: vec![
                    ("Removing copy", vec![item("a.pdf"), item("b.pdf")]),
                    ("Removing copies", vec![item("c.pdf")]),
                ],
                summary: Summary::Files("Removed duplicates of"),
            },
        };
        let outcome = |call: &Call| match call {
            Call::TrashDuplicateFile(change) if change.path.value() == "b.pdf" => OperationOutcome::NotFound,
            _ => done(),
        };
        execute(&session, plan, outcome, |_| {}).expect("started");
        assert_eq!(messages_after(&recorder, &finished), vec![UiMessage::error("Removed duplicates of 2 of 3 files")]);
    }

    #[test]
    fn a_busy_lane_refuses_the_plan() {
        let (session, _, _finished) = session();
        let (release, gate) = channel::<()>();
        assert!(session.perform("Scanning", Lane::Library, false, move |_| gate.recv().map_err(|e| e.to_string())));
        let plan = ActionPlan {
            label: "Dismissing the task".to_owned(),
            lane: Lane::Library,
            cancellable: false,
            work: Work::One {
                call: Call::DismissTask(dms_domain::task::TaskKey::new("k")),
                done: None,
                resolves: None,
            },
        };
        let error = execute(&session, plan, |_| done(), |_| {}).expect_err("busy");
        assert_eq!(error.code, "busy");
        release.send(()).expect("release");
    }
}
