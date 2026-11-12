//! What a dashboard action does: the operation's label and lane, the use cases it calls and how its outcome is
//! reported (the `act`/`perform` calls of Kotlin's `DashboardViewModel`, `BulkActions` and `OrganizeTasks`).
//! Planning is pure and works on the current dashboard; [`super::run`] executes a plan in the session.

use dms_application::library::Dashboard;
use dms_application::resolution::{DuplicatePreference, MoveDecision};
use dms_domain::cases::{CaseId, CaseStatus, FollowUpId};
use dms_domain::catalog::Catalog;
use dms_domain::change::{DetectedChange, MissingFile, MovedFile, NewFile};
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::{LibraryFolderId, RelativePath};
use dms_domain::rule::RuleProposal;
use dms_domain::task::{OpenTask, TaskKey};

use super::dto::{DashboardAction, MoveChoice, TaskGroupKind};
use super::view::context::TaskContext;
use super::view::groups;
use crate::dto::IpcError;
use crate::session::Lane;

/// How far "Snooze 7 days" moves a follow-up.
const SNOOZE_DAYS: u16 = 7;

/// One call of a use case, with the domain values it takes. Each variant names its use case.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    /// `services.documents.adopt_new_file.run(&change)`
    AdoptNewFile(NewFile),
    /// `services.documents.restore_document.run(&change)`
    RestoreDocument(MissingFile),
    /// `services.documents.trash_new_file.run(&change, true)`
    TrashNewFile(NewFile),
    /// `services.documents.trash_duplicate_file.run(&change, true)`
    TrashDuplicateFile(NewFile),
    /// `services.documents.trash_document.run(&id, true)`
    TrashDocument(DocumentId),
    /// `services.sync.keep_version.run(&path, &source, true)`
    KeepVersion { path: RelativePath, source: LibraryFolderId },
    /// `services.sync.keep_both_versions.run(&path, &folder)`
    KeepBothVersions { path: RelativePath, folder: LibraryFolderId },
    /// `services.sync.resolve_move.run(&change, decision)`
    ResolveMove { change: MovedFile, decision: MoveDecision },
    /// `services.sync.merge_duplicates.run(&keep, true)`
    MergeDuplicates(DocumentId),
    /// The preferred of these copies in the catalog at run time, then `merge_duplicates`; fails for fewer than two.
    MergePreferredOf(Vec<DocumentId>),
    /// `services.ai.reject_suggestion.run(&id)`
    RejectSuggestion(DocumentId),
    /// `services.sync.dismiss_task.run(&key)`
    DismissTask(TaskKey),
    /// `services.documents.settle_retention.run(&ids)`
    SettleRetention(Vec<DocumentId>),
    /// `services.organize.accept_rule_proposal.run(&proposal)`
    AcceptRuleProposal(RuleProposal),
    /// `services.organize.dismiss_contract_proposal.run(&correspondent)`
    DismissContractProposal(String),
    /// `services.cases.complete_follow_up.run(&case_id, &follow_up_id)`
    CompleteFollowUp { case_id: CaseId, follow_up_id: FollowUpId },
    /// `services.cases.snooze_follow_up.run(&case_id, &follow_up_id, days)`
    SnoozeFollowUp { case_id: CaseId, follow_up_id: FollowUpId, days: u16 },
    /// `services.cases.set_case_status.run(&case_id, status)`
    SetCaseStatus { case_id: CaseId, status: CaseStatus },
    /// `services.sync.repair_copy.run(&document_id, &folder_id, &source_folder_id, true)`
    RepairCopy { document_id: DocumentId, folder_id: LibraryFolderId, source_folder_id: LibraryFolderId },
}

/// One item of a run over many: its name in progress and log, the call, and the change it resolves.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub call: Call,
    pub resolves: Option<DetectedChange>,
}

/// How a run over many items reports: "{verb} {done} of {total} files" (an error when not all succeeded), or
/// "Moved {done} of {total} documents to the trash".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Summary {
    Files(&'static str),
    TrashedDocuments,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // one plan per click
pub enum Work {
    /// One call; `done` is the success text of its outcome message, `None` reports nothing. A `resolves` change is
    /// forgotten from the last scan once the call is done, so its task disappears without a new scan.
    One { call: Call, done: Option<String>, resolves: Option<DetectedChange> },
    /// Calls one after the other with progress, phase by phase; each phase has its progress step.
    Each { phases: Vec<(&'static str, Vec<Item>)>, summary: Summary },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionPlan {
    pub label: String,
    pub lane: Lane,
    pub cancellable: bool,
    pub work: Work,
}

impl ActionPlan {
    fn one(label: String, call: Call, done: Option<String>, resolves: Option<DetectedChange>) -> Self {
        Self { label, lane: Lane::Library, cancellable: false, work: Work::One { call, done, resolves } }
    }

    fn each(label: String, cancellable: bool, phases: Vec<(&'static str, Vec<Item>)>, summary: Summary) -> Self {
        Self { label, lane: Lane::Library, cancellable, work: Work::Each { phases, summary } }
    }
}

/// The plan of `action` against the current `dashboard`.
///
/// # Errors
/// `invalid_input` for actions the window handles itself, unknown ids, or a task that is gone.
pub fn plan(action: &DashboardAction, dashboard: &Dashboard, catalog: &Catalog) -> Result<ActionPlan, IpcError> {
    let context = TaskContext::new(dashboard.folders.iter().map(|state| &state.folder).collect(), catalog);
    match action {
        DashboardAction::Adopt { .. }
        | DashboardAction::TrashNewFile { .. }
        | DashboardAction::TrashDuplicateFile { .. }
        | DashboardAction::Restore { .. }
        | DashboardAction::DeleteEverywhere { .. } => plan_file(action, dashboard),
        DashboardAction::SettleRetention { .. }
        | DashboardAction::TrashRetained { .. }
        | DashboardAction::AdoptAll
        | DashboardAction::RestoreAll
        | DashboardAction::RenameAll
        | DashboardAction::RemoveAllDuplicates => plan_bulk(action, dashboard, &context),
        DashboardAction::CompleteFollowUp { .. }
        | DashboardAction::SnoozeFollowUp { .. }
        | DashboardAction::CloseCase { .. } => plan_case(action, dashboard),
        DashboardAction::DismissContractProposal { .. } | DashboardAction::RepairCopy { .. } => {
            plan_feature(action, dashboard, &context)
        }
        _ => plan_resolution(action, dashboard, &context),
    }
}

/// Completing or snoozing a follow-up, closing a case.
fn plan_case(action: &DashboardAction, dashboard: &Dashboard) -> Result<ActionPlan, IpcError> {
    let find = |key: &str| task(dashboard, key);
    match action {
        DashboardAction::CompleteFollowUp { task_key } => {
            let OpenTask::FollowUpDue { case_id, follow_up_id, .. } = find(task_key)? else { return Err(wrong_task()) };
            let call = Call::CompleteFollowUp { case_id: case_id.clone(), follow_up_id: follow_up_id.clone() };
            Ok(ActionPlan::one("Completing the follow-up".to_owned(), call, None, None))
        }
        DashboardAction::SnoozeFollowUp { task_key } => {
            let OpenTask::FollowUpDue { case_id, follow_up_id, .. } = find(task_key)? else { return Err(wrong_task()) };
            let call = Call::SnoozeFollowUp {
                case_id: case_id.clone(),
                follow_up_id: follow_up_id.clone(),
                days: SNOOZE_DAYS,
            };
            Ok(ActionPlan::one(
                "Snoozing the follow-up".to_owned(),
                call,
                Some(format!("Snoozed for {SNOOZE_DAYS} days")),
                None,
            ))
        }
        DashboardAction::CloseCase { task_key } => {
            let OpenTask::CaseWaitingTooLong { case_id, .. } = find(task_key)? else { return Err(wrong_task()) };
            let call = Call::SetCaseStatus { case_id: case_id.clone(), status: CaseStatus::Done };
            Ok(ActionPlan::one("Closing the case".to_owned(), call, Some("Marked the case as done".to_owned()), None))
        }
        _ => Err(IpcError::invalid("The window handles this action itself")),
    }
}

/// Adopting, restoring and trashing one file.
fn plan_file(action: &DashboardAction, dashboard: &Dashboard) -> Result<ActionPlan, IpcError> {
    let find = |key: &str| task(dashboard, key);
    match action {
        DashboardAction::Adopt { task_key } => {
            let change = new_file(find(task_key)?)?;
            Ok(ActionPlan::one(
                format!("Adopting {}", change.path),
                Call::AdoptNewFile(change.clone()),
                Some(format!("Adopted {}", change.path)),
                Some(DetectedChange::NewFile(change.clone())),
            ))
        }
        DashboardAction::TrashNewFile { task_key } => {
            let change = new_file(find(task_key)?)?;
            Ok(ActionPlan::one(
                format!("Trashing {}", change.path),
                Call::TrashNewFile(change.clone()),
                Some(format!("Moved {} to the trash", change.path)),
                Some(DetectedChange::NewFile(change.clone())),
            ))
        }
        DashboardAction::TrashDuplicateFile { task_key } => {
            let change = new_file(find(task_key)?)?;
            Ok(ActionPlan::one(
                format!("Trashing {}", change.path),
                Call::TrashDuplicateFile(change.clone()),
                Some(format!("Moved the copy {} to the trash", change.path)),
                Some(DetectedChange::NewFile(change.clone())),
            ))
        }
        DashboardAction::Restore { task_key } => {
            let OpenTask::RestoreOrDelete(change) = find(task_key)? else { return Err(wrong_task()) };
            Ok(ActionPlan::one(
                format!("Restoring {}", change.path),
                Call::RestoreDocument(change.clone()),
                Some(format!("Restored {}", change.path)),
                Some(DetectedChange::MissingFile(change.clone())),
            ))
        }
        DashboardAction::DeleteEverywhere { task_key } => {
            let OpenTask::RestoreOrDelete(change) = find(task_key)? else { return Err(wrong_task()) };
            Ok(ActionPlan::one(
                format!("Deleting {}", change.path),
                Call::TrashDocument(change.document_id.clone()),
                Some(format!("Moved {} to the trash", change.path)),
                Some(DetectedChange::MissingFile(change.clone())),
            ))
        }
        _ => Err(IpcError::invalid("The window handles this action itself")),
    }
}

/// Deciding a task: versions, moves, duplicates, suggestions, rules, dismissing.
fn plan_resolution(
    action: &DashboardAction,
    dashboard: &Dashboard,
    context: &TaskContext<'_>,
) -> Result<ActionPlan, IpcError> {
    let catalog = context.catalog();
    let find = |key: &str| task(dashboard, key);
    match action {
        DashboardAction::KeepVersion { task_key, folder_id } => {
            let source = folder(folder_id)?;
            let change = match find(task_key)? {
                OpenTask::ReviewModification(change) => DetectedChange::ModifiedFile(change.clone()),
                OpenTask::ResolveContentConflict(change) => DetectedChange::ContentConflict(change.clone()),
                _ => return Err(wrong_task()),
            };
            let name = context.folder_name(&source);
            Ok(ActionPlan::one(
                format!("Keeping the version of {name}"),
                Call::KeepVersion { path: change.path().clone(), source },
                Some(format!("Kept the version of {name}")),
                Some(change),
            ))
        }
        DashboardAction::KeepBoth { task_key, folder_id } => {
            let folder = folder(folder_id)?;
            let OpenTask::ResolveContentConflict(change) = find(task_key)? else { return Err(wrong_task()) };
            Ok(ActionPlan::one(
                "Keeping both versions".to_owned(),
                Call::KeepBothVersions { path: change.path.clone(), folder },
                Some(format!("Kept both versions of {}", change.path)),
                Some(DetectedChange::ContentConflict(change.clone())),
            ))
        }
        DashboardAction::ResolveMove { task_key, choice } => {
            let OpenTask::ConfirmMove(change) = find(task_key)? else { return Err(wrong_task()) };
            let (decision, done) = match choice {
                MoveChoice::Apply => {
                    (MoveDecision::Apply, format!("Moved {} to {} everywhere", change.previous_path, change.path))
                }
                MoveChoice::Undo => {
                    (MoveDecision::Undo, format!("Moved {} back to {}", change.path, change.previous_path))
                }
            };
            Ok(ActionPlan::one(
                format!("Resolving the move of {}", change.path),
                Call::ResolveMove { change: change.clone(), decision },
                Some(done),
                Some(DetectedChange::MovedFile(change.clone())),
            ))
        }
        DashboardAction::KeepDuplicate { task_key, document_id } => {
            let OpenTask::ResolveDuplicates { document_ids, .. } = find(task_key)? else { return Err(wrong_task()) };
            let keep = document(document_id)?;
            let kept = catalog.documents().get(&keep).ok_or_else(|| IpcError::invalid("The document is gone"))?;
            let copies = context.documents(document_ids).iter().filter(|copy| copy.id != keep).count();
            Ok(ActionPlan::one(
                format!("Removing copies of {}", kept.path),
                Call::MergeDuplicates(keep),
                Some(format!("Kept {}; moved {copies} copies to the trash", kept.path)),
                None,
            ))
        }
        DashboardAction::Reject { document_id } => Ok(ActionPlan::one(
            "Rejecting the suggestion".to_owned(),
            Call::RejectSuggestion(document(document_id)?),
            None,
            None,
        )),
        DashboardAction::Dismiss { task_key } => {
            Ok(ActionPlan::one("Dismissing the task".to_owned(), Call::DismissTask(find(task_key)?.key()), None, None))
        }
        DashboardAction::AcceptRule { task_key } => {
            let OpenTask::ReviewLearnedRule(proposal) = find(task_key)? else { return Err(wrong_task()) };
            Ok(ActionPlan::one(
                format!("Adding the rule for {}", proposal.correspondent),
                Call::AcceptRuleProposal(proposal.clone()),
                Some(format!("Documents of {} now get the rule's values proposed", proposal.correspondent)),
                None,
            ))
        }
        _ => Err(IpcError::invalid("The window handles this action itself")),
    }
}

/// Answering a task of a 2.1 feature: a contract proposal, a damaged copy.
fn plan_feature(
    action: &DashboardAction,
    dashboard: &Dashboard,
    context: &TaskContext<'_>,
) -> Result<ActionPlan, IpcError> {
    let find = |key: &str| task(dashboard, key);
    match action {
        DashboardAction::DismissContractProposal { task_key } => {
            let OpenTask::ReviewContractProposal(proposal) = find(task_key)? else { return Err(wrong_task()) };
            Ok(ActionPlan::one(
                "Dismissing the contract proposal".to_owned(),
                Call::DismissContractProposal(proposal.correspondent.clone()),
                Some(format!("No contract is proposed for {} again", proposal.correspondent)),
                None,
            ))
        }
        DashboardAction::RepairCopy { task_key } => {
            let OpenTask::DamagedCopy { document_id, folder_id, source_folder_id } = find(task_key)? else {
                return Err(wrong_task());
            };
            let (path, folder) = (context.document_path(document_id), context.folder_name(folder_id));
            Ok(ActionPlan::one(
                format!("Repairing {path}"),
                Call::RepairCopy {
                    document_id: document_id.clone(),
                    folder_id: folder_id.clone(),
                    source_folder_id: source_folder_id.clone(),
                },
                Some(format!("Repaired the copy of {path} in {folder}")),
                None,
            ))
        }
        _ => Err(IpcError::invalid("The window handles this action itself")),
    }
}

/// Answering many documents or a whole group at once.
fn plan_bulk(
    action: &DashboardAction,
    dashboard: &Dashboard,
    context: &TaskContext<'_>,
) -> Result<ActionPlan, IpcError> {
    match action {
        DashboardAction::SettleRetention { document_ids } => {
            let ids = documents(document_ids)?;
            Ok(ActionPlan::one(
                "Marking the retention as done".to_owned(),
                Call::SettleRetention(ids),
                Some(format!("Kept {} documents; their retention is done", document_ids.len())),
                None,
            ))
        }
        DashboardAction::TrashRetained { document_ids } => {
            let ids = documents(document_ids)?;
            let items = ids
                .into_iter()
                .map(|id| Item { name: context.document_path(&id), call: Call::TrashDocument(id), resolves: None })
                .collect();
            Ok(ActionPlan::each(
                format!("Moving {} documents to the trash", paths_known(document_ids, context)),
                true,
                vec![("Moving to trash", items)],
                Summary::TrashedDocuments,
            ))
        }
        DashboardAction::AdoptAll => {
            let tasks = group(dashboard, context, TaskGroupKind::NewFiles);
            let items: Vec<Item> = groups::adoptable(&tasks)
                .into_iter()
                .map(|change| file_item(change, Call::AdoptNewFile(change.clone())))
                .collect();
            Ok(ActionPlan::each(
                format!("Adopting {} new files", items.len()),
                true,
                vec![("Adopting", items)],
                Summary::Files("Adopted"),
            ))
        }
        DashboardAction::RestoreAll => {
            let items: Vec<Item> = group(dashboard, context, TaskGroupKind::Missing)
                .into_iter()
                .filter_map(|task| match task {
                    OpenTask::RestoreOrDelete(change) if !change.present_in.is_empty() => Some(Item {
                        name: change.path.to_string(),
                        call: Call::RestoreDocument(change.clone()),
                        resolves: Some(DetectedChange::MissingFile(change.clone())),
                    }),
                    _ => None,
                })
                .collect();
            Ok(ActionPlan::each(
                format!("Restoring {} files", items.len()),
                false,
                vec![("Restoring", items)],
                Summary::Files("Restored"),
            ))
        }
        DashboardAction::RenameAll => {
            let items: Vec<Item> = group(dashboard, context, TaskGroupKind::Moved)
                .into_iter()
                .filter_map(|task| match task {
                    OpenTask::ConfirmMove(change) => Some(Item {
                        name: change.path.to_string(),
                        call: Call::ResolveMove { change: change.clone(), decision: MoveDecision::Undo },
                        resolves: Some(DetectedChange::MovedFile(change.clone())),
                    }),
                    _ => None,
                })
                .collect();
            Ok(ActionPlan::each(
                format!("Renaming {} files to match", items.len()),
                true,
                vec![("Renaming", items)],
                Summary::Files("Renamed"),
            ))
        }
        DashboardAction::RemoveAllDuplicates => {
            let tasks = group(dashboard, context, TaskGroupKind::Duplicates);
            let files: Vec<Item> = groups::duplicate_files(&tasks)
                .into_iter()
                .map(|change| file_item(change, Call::TrashDuplicateFile(change.clone())))
                .collect();
            let merges: Vec<Item> = groups::catalogued_duplicates(&tasks)
                .into_iter()
                .map(|ids| Item {
                    name: ids.first().map_or_else(String::new, |id| context.document_path(id)),
                    call: Call::MergePreferredOf(ids.to_vec()),
                    resolves: None,
                })
                .collect();
            Ok(ActionPlan::each(
                "Removing duplicates".to_owned(),
                true,
                vec![("Removing copy", files), ("Removing copies", merges)],
                Summary::Files("Removed duplicates of"),
            ))
        }
        _ => Err(IpcError::invalid("The window handles this action itself")),
    }
}

/// The copy to keep of `ids` in `catalog` (Kotlin `BulkActions.merge`): `None` when fewer than two are left.
pub fn preferred_of(ids: &[DocumentId], catalog: &Catalog) -> Option<DocumentId> {
    let copies: Vec<Document> = ids.iter().filter_map(|id| catalog.documents().get(id).cloned()).collect();
    if copies.len() < 2 {
        return None;
    }
    DuplicatePreference::preferred(&copies).map(|document| document.id.clone())
}

fn file_item(change: &NewFile, call: Call) -> Item {
    Item { name: change.path.to_string(), call, resolves: Some(DetectedChange::NewFile(change.clone())) }
}

fn group<'d>(dashboard: &'d Dashboard, context: &TaskContext<'_>, group: TaskGroupKind) -> Vec<&'d OpenTask> {
    dashboard.tasks.iter().filter(|task| groups::group_of(task, context) == group).collect()
}

/// Kotlin's label counted the paths the catalog still knows.
fn paths_known(ids: &[String], context: &TaskContext<'_>) -> usize {
    ids.iter().filter_map(|id| DocumentId::new(id.as_str()).ok()).filter(|id| context.path_of(id).is_some()).count()
}

fn task<'d>(dashboard: &'d Dashboard, key: &str) -> Result<&'d OpenTask, IpcError> {
    dashboard
        .tasks
        .iter()
        .find(|task| task.key().value() == key)
        .ok_or_else(|| IpcError::invalid("The task is no longer open"))
}

fn new_file(task: &OpenTask) -> Result<&NewFile, IpcError> {
    match task {
        OpenTask::AdoptNewFile(change) => Ok(change),
        _ => Err(wrong_task()),
    }
}

fn wrong_task() -> IpcError {
    IpcError::invalid("The action does not fit the task")
}

fn folder(id: &str) -> Result<LibraryFolderId, IpcError> {
    LibraryFolderId::new(id).map_err(|error| IpcError::invalid(error.to_string()))
}

fn document(id: &str) -> Result<DocumentId, IpcError> {
    DocumentId::new(id).map_err(|error| IpcError::invalid(error.to_string()))
}

fn documents(ids: &[String]) -> Result<Vec<DocumentId>, IpcError> {
    ids.iter().map(|id| document(id)).collect()
}
