//! Which group lists a task, how the groups look, how urgent they are, and the buttons that answer a whole group
//! (Kotlin `TaskGroup`, `TaskGroups`, `AttentionRows.severityOf`, `BulkActions.buttonsFor`,
//! `OrganizeTasks.bulkButtonsFor`).

use dms_domain::change::NewFile;
use dms_domain::document::DocumentId;
use dms_domain::task::OpenTask;

use super::actions::{remove_all_duplicates, trash_retained};
use super::buttons::{ButtonExt, button};
use super::context::TaskContext;
use crate::features::dashboard::dto::{AttentionSeverity, DashboardAction, DashboardButtonDto, TaskGroupKind};
use crate::session::Lane;

/// Every group in the order the dashboard lists them.
pub const ORDER: [TaskGroupKind; 13] = [
    TaskGroupKind::Suggestions,
    TaskGroupKind::Deadlines,
    TaskGroupKind::Retention,
    TaskGroupKind::LearnedRules,
    TaskGroupKind::NewFiles,
    TaskGroupKind::Missing,
    TaskGroupKind::Modified,
    TaskGroupKind::Moved,
    TaskGroupKind::Duplicates,
    TaskGroupKind::Conflicts,
    TaskGroupKind::CatalogMerge,
    TaskGroupKind::FailedOperations,
    TaskGroupKind::Restructure,
];

/// The group of `task`; a new file that copies a document is a duplicate.
pub fn group_of(task: &OpenTask, context: &TaskContext<'_>) -> TaskGroupKind {
    match task {
        OpenTask::ReviewSuggestion(_) => TaskGroupKind::Suggestions,
        OpenTask::AdoptNewFile(change) => {
            if context.original_of(change).is_none() {
                TaskGroupKind::NewFiles
            } else {
                TaskGroupKind::Duplicates
            }
        }
        OpenTask::RestoreOrDelete(_) => TaskGroupKind::Missing,
        OpenTask::ReviewModification(_) => TaskGroupKind::Modified,
        OpenTask::ConfirmMove(_) => TaskGroupKind::Moved,
        OpenTask::ResolveDuplicates { .. } => TaskGroupKind::Duplicates,
        OpenTask::ResolveContentConflict(_) => TaskGroupKind::Conflicts,
        OpenTask::ReviewCatalogMerge(_) => TaskGroupKind::CatalogMerge,
        OpenTask::RetryFailedOperations { .. } => TaskGroupKind::FailedOperations,
        OpenTask::DeadlineDue { .. } => TaskGroupKind::Deadlines,
        OpenTask::RetentionDue { .. } => TaskGroupKind::Retention,
        OpenTask::RestructureSuggested { .. } => TaskGroupKind::Restructure,
        OpenTask::ReviewLearnedRule(_) => TaskGroupKind::LearnedRules,
    }
}

pub fn label(group: TaskGroupKind) -> &'static str {
    match group {
        TaskGroupKind::Suggestions => "Suggestions",
        TaskGroupKind::Deadlines => "Deadlines",
        TaskGroupKind::Retention => "Retention ended",
        TaskGroupKind::LearnedRules => "Learned rules",
        TaskGroupKind::NewFiles => "New files",
        TaskGroupKind::Missing => "Missing files",
        TaskGroupKind::Modified => "Changed outside",
        TaskGroupKind::Moved => "Different names",
        TaskGroupKind::Duplicates => "Duplicates",
        TaskGroupKind::Conflicts => "Conflicts",
        TaskGroupKind::CatalogMerge => "Catalog merge",
        TaskGroupKind::FailedOperations => "Failed operations",
        TaskGroupKind::Restructure => "Folder structure",
    }
}

pub fn icon(group: TaskGroupKind) -> &'static str {
    match group {
        TaskGroupKind::Suggestions => "auto_awesome",
        TaskGroupKind::Deadlines => "event",
        TaskGroupKind::Retention => "inventory_2",
        TaskGroupKind::LearnedRules => "rule",
        TaskGroupKind::NewFiles => "new_releases",
        TaskGroupKind::Missing => "find_in_page",
        TaskGroupKind::Modified => "edit",
        TaskGroupKind::Moved => "swap_horiz",
        TaskGroupKind::Duplicates => "content_copy",
        TaskGroupKind::Conflicts => "error_outline",
        TaskGroupKind::CatalogMerge => "merge_type",
        TaskGroupKind::FailedOperations => "sync_problem",
        TaskGroupKind::Restructure => "account_tree",
    }
}

/// Dates that passed first, then what needs a decision, then the rest.
pub fn severity_of(group: TaskGroupKind) -> AttentionSeverity {
    match group {
        TaskGroupKind::Deadlines | TaskGroupKind::Retention => AttentionSeverity::Passed,
        TaskGroupKind::Missing
        | TaskGroupKind::Modified
        | TaskGroupKind::Moved
        | TaskGroupKind::Duplicates
        | TaskGroupKind::Conflicts
        | TaskGroupKind::CatalogMerge
        | TaskGroupKind::FailedOperations => AttentionSeverity::Soon,
        TaskGroupKind::Suggestions
        | TaskGroupKind::LearnedRules
        | TaskGroupKind::NewFiles
        | TaskGroupKind::Restructure => AttentionSeverity::Info,
    }
}

/// The buttons that answer every task of `group` at once; they wait for the library lane.
pub fn bulk_buttons(group: TaskGroupKind, tasks: &[&OpenTask], context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let buttons = match group {
        TaskGroupKind::NewFiles => {
            vec![button(format!("Adopt all ({})", adoptable(tasks).len()), DashboardAction::AdoptAll)]
        }
        TaskGroupKind::Missing => {
            vec![button(format!("Restore all ({})", restorable(tasks)), DashboardAction::RestoreAll)]
        }
        TaskGroupKind::Moved => {
            let moved = tasks.iter().filter(|task| matches!(task, OpenTask::ConfirmMove(_))).count();
            vec![button(format!("Rename all to match ({moved})"), DashboardAction::RenameAll)]
        }
        TaskGroupKind::Duplicates => {
            let files = duplicate_files(tasks);
            let catalogued = catalogued_duplicates(tasks);
            vec![
                button(
                    format!("Remove all duplicates ({})…", files.len() + catalogued.len()),
                    DashboardAction::RemoveAllDuplicates,
                )
                .confirm(remove_all_duplicates(&files, &catalogued, context)),
            ]
        }
        TaskGroupKind::Retention => {
            let ids = retained(tasks);
            let names: Vec<String> = ids.iter().map(ToString::to_string).collect();
            vec![
                button(
                    format!("Mark all as done ({})", ids.len()),
                    DashboardAction::SettleRetention { document_ids: names.clone() },
                ),
                button(
                    format!("Move all to trash ({})…", ids.len()),
                    DashboardAction::TrashRetained { document_ids: names },
                )
                .confirm(trash_retained(&ids, context)),
            ]
        }
        TaskGroupKind::Suggestions
        | TaskGroupKind::Deadlines
        | TaskGroupKind::LearnedRules
        | TaskGroupKind::Modified
        | TaskGroupKind::Conflicts
        | TaskGroupKind::CatalogMerge
        | TaskGroupKind::FailedOperations
        | TaskGroupKind::Restructure => Vec::new(),
    };
    buttons.into_iter().map(|button| button.waits_for(Lane::Library)).collect()
}

/// The readable new files, one per content: of several files with the same content only one is adopted.
pub fn adoptable<'t>(tasks: &[&'t OpenTask]) -> Vec<&'t NewFile> {
    let mut seen = Vec::new();
    let mut files = Vec::new();
    for task in tasks {
        if let OpenTask::AdoptNewFile(change) = task
            && let Some(hash) = &change.content_hash
            && !seen.contains(&hash)
        {
            seen.push(hash);
            files.push(change);
        }
    }
    files
}

fn restorable(tasks: &[&OpenTask]) -> usize {
    tasks
        .iter()
        .filter(|task| matches!(task, OpenTask::RestoreOrDelete(change) if !change.present_in.is_empty()))
        .count()
}

pub fn duplicate_files<'t>(tasks: &[&'t OpenTask]) -> Vec<&'t NewFile> {
    tasks
        .iter()
        .filter_map(|task| if let OpenTask::AdoptNewFile(change) = task { Some(change) } else { None })
        .collect()
}

pub fn catalogued_duplicates<'t>(tasks: &[&'t OpenTask]) -> Vec<&'t [DocumentId]> {
    tasks
        .iter()
        .filter_map(|task| match task {
            OpenTask::ResolveDuplicates { document_ids, .. } => Some(document_ids.as_slice()),
            _ => None,
        })
        .collect()
}

fn retained(tasks: &[&OpenTask]) -> Vec<DocumentId> {
    tasks
        .iter()
        .filter_map(|task| match task {
            OpenTask::RetentionDue { document_id, .. } => Some(document_id.clone()),
            _ => None,
        })
        .collect()
}
