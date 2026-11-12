//! The buttons of every kind of task: exactly the actions its use cases support, the files it is about and the
//! questions asked before destructive actions (Kotlin `TaskActions` and the dialogs of `DashboardViewModel`).

use dms_application::resolution::DuplicatePreference;
use dms_domain::change::{ContentConflict, MissingFile, ModifiedFile, NewFile};
use dms_domain::document::{Document, DocumentId};
use dms_domain::expectation::ExpectedDocument;
use dms_domain::iso_date;
use dms_domain::library::{LibraryFolderId, RelativePath};
use dms_domain::task::OpenTask;

use super::buttons::{ButtonExt, button, confirmation};
use super::context::TaskContext;
use super::groups::{group_of, screen_of};
use crate::features::bills::view::expected_title;
use crate::features::cases::view as cases;
use crate::features::dashboard::dto::{
    DashboardAction, DashboardButtonDto, DashboardButtonStyle, DashboardConfirmationDto, DuplicateCopyDto, MoveChoice,
    TaskFileDto,
};
use crate::session::Lane;

/// A file a task is about and the label of the button that opens it.
pub struct TaskFile {
    pub label: String,
    pub file: TaskFileDto,
}

/// The action buttons of `task`; each waits for the library lane.
pub fn buttons_for(task: &OpenTask, context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let key = task.key().value().to_owned();
    let dismiss =
        || button("Dismiss", DashboardAction::Dismiss { task_key: key.clone() }).style(DashboardButtonStyle::Text);
    let buttons = match task {
        OpenTask::ReviewSuggestion(suggestion) => {
            let document_id = suggestion.document_id.to_string();
            vec![
                button("Review", DashboardAction::Review { document_id: document_id.clone() })
                    .icon("auto_awesome")
                    .style(DashboardButtonStyle::Tonal),
                button("Reject", DashboardAction::Reject { document_id }).style(DashboardButtonStyle::Text),
            ]
        }
        OpenTask::AdoptNewFile(change) => with(new_file_buttons(&key, change, context), dismiss()),
        OpenTask::RestoreOrDelete(change) => vec![
            button("Restore", DashboardAction::Restore { task_key: key.clone() })
                .icon("restore")
                .style(DashboardButtonStyle::Tonal)
                .disabled(change.present_in.is_empty().then_some("No reachable folder still has this file")),
            button("Move to trash…", DashboardAction::DeleteEverywhere { task_key: key.clone() })
                .confirm(delete_everywhere(change, context)),
            dismiss(),
        ],
        OpenTask::ReviewModification(change) => with(modification_buttons(&key, change, context), dismiss()),
        OpenTask::ConfirmMove(change) => vec![
            button(
                format!("Rename to {} here", change.previous_path.file_name()),
                DashboardAction::ResolveMove { task_key: key.clone(), choice: MoveChoice::Undo },
            )
            .icon("sync")
            .style(DashboardButtonStyle::Tonal),
            button(
                format!("Use {} everywhere", change.path.file_name()),
                DashboardAction::ResolveMove { task_key: key.clone(), choice: MoveChoice::Apply },
            ),
            dismiss(),
        ],
        OpenTask::ResolveDuplicates { .. } | OpenTask::DeadlineDue { .. } | OpenTask::UnreadableDocument { .. } => {
            vec![dismiss()]
        }
        OpenTask::DamagedCopy { document_id, folder_id, source_folder_id } => vec![
            button(
                format!("Repair from {}…", context.folder_name(source_folder_id)),
                DashboardAction::RepairCopy { task_key: key.clone() },
            )
            .icon("healing")
            .style(DashboardButtonStyle::Tonal)
            .confirm(repair_copy(document_id, folder_id, source_folder_id, context)),
            dismiss(),
        ],
        OpenTask::ResolveContentConflict(change) => with(conflict_buttons(&key, change, context), dismiss()),
        OpenTask::ReviewCatalogMerge(_) => vec![button("OK", DashboardAction::Dismiss { task_key: key.clone() })],
        OpenTask::RetryFailedOperations { .. } => {
            vec![button("Retry", DashboardAction::Retry).icon("refresh").style(DashboardButtonStyle::Tonal), dismiss()]
        }
        OpenTask::RetentionDue { document_id, .. } => retention_buttons(std::slice::from_ref(document_id), context),
        OpenTask::RestructureSuggested { folder, .. } => vec![
            button("Restructure…", DashboardAction::Restructure { folder: folder.to_string() })
                .icon("account_tree")
                .style(DashboardButtonStyle::Tonal),
            dismiss(),
        ],
        OpenTask::ReviewLearnedRule(_) => vec![
            button("Accept rule", DashboardAction::AcceptRule { task_key: key.clone() })
                .icon("check")
                .style(DashboardButtonStyle::Tonal),
            button("Reject", DashboardAction::Dismiss { task_key: key.clone() }).style(DashboardButtonStyle::Text),
        ],
        OpenTask::BillOverdue { document_id, .. } => bill_buttons(document_id),
        OpenTask::ExpectedDocumentMissing(expected) => expected_buttons(&key, expected, context),
        OpenTask::ReviewContractProposal(proposal) => vec![
            button(
                "Create contract",
                DashboardAction::CreateContract {
                    correspondent: proposal.correspondent.clone(),
                    document_ids: proposal.document_ids.iter().map(ToString::to_string).collect(),
                },
            )
            .icon("add")
            .style(DashboardButtonStyle::Tonal),
            button("Dismiss", DashboardAction::DismissContractProposal { task_key: key.clone() })
                .style(DashboardButtonStyle::Text),
        ],
        OpenTask::ContractCancelBy { contract_id, .. } => {
            let open =
                |draft_letter| DashboardAction::OpenContract { contract_id: contract_id.to_string(), draft_letter };
            vec![
                button("Open", open(false)).icon("arrow_forward").style(DashboardButtonStyle::Tonal),
                button("Draft cancellation letter", open(true)).icon("edit_note"),
                dismiss(),
            ]
        }
        OpenTask::FollowUpDue { case_id, .. } => cases::follow_up_buttons(case_id, &key),
        OpenTask::CaseWaitingTooLong { case_id, .. } => cases::waiting_buttons(case_id, &key, context.catalog()),
        OpenTask::EmergencyFolderRefresh { .. } => {
            screen_of(group_of(task, context)).map(open_screen).into_iter().chain([dismiss()]).collect()
        }
    };
    buttons.into_iter().map(|button| button.waits_for(Lane::Library)).collect()
}

/// "Mark as paid" and "QR code" for an overdue bill; "Open" comes with the files.
fn bill_buttons(document_id: &DocumentId) -> Vec<DashboardButtonDto> {
    let document_id = document_id.to_string();
    vec![
        button("Mark as paid", DashboardAction::MarkBillPaid { document_id: document_id.clone() })
            .icon("check")
            .style(DashboardButtonStyle::Tonal),
        button("QR code", DashboardAction::ShowPaymentCode { document_id }).icon("qr_code_2"),
    ]
}

/// "Requested", "Ignore once", "Stop expecting…" and "Draft request" for a missing document.
fn expected_buttons(key: &str, expected: &ExpectedDocument, context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let task_key = key.to_owned();
    let what = expected_title(expected, context.catalog());
    let draft = DashboardAction::DraftRequest {
        recipient: expected.correspondent.clone(),
        document_ids: Vec::new(),
        subject_facts: vec![format!("{what} expected around {}, not received", iso_date(expected.expected_on))],
    };
    vec![
        button("Requested", DashboardAction::ExpectedRequested { task_key: task_key.clone() })
            .icon("schedule_send")
            .style(DashboardButtonStyle::Tonal),
        button("Ignore once", DashboardAction::Dismiss { task_key: task_key.clone() }),
        button("Stop expecting…", DashboardAction::StopExpecting { task_key }).confirm(confirmation(
            "Stop expecting".to_owned(),
            format!("No more reminders when documents from {} ({what}) are late.", expected.correspondent),
            Vec::new(),
            "Stop expecting",
        )),
        button("Draft request", draft).icon("edit_note").style(DashboardButtonStyle::Text),
    ]
}

/// "Open" for the screen of a feature task; it only navigates.
fn open_screen(screen: crate::features::dashboard::dto::DashboardScreenDto) -> DashboardButtonDto {
    button("Open", DashboardAction::OpenScreen { screen }).icon("arrow_forward").style(DashboardButtonStyle::Tonal)
}

fn with(mut buttons: Vec<DashboardButtonDto>, last: DashboardButtonDto) -> Vec<DashboardButtonDto> {
    buttons.push(last);
    buttons
}

/// "Mark as done" and "Move to trash…" for documents whose retention ended.
pub fn retention_buttons(document_ids: &[DocumentId], context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let ids: Vec<String> = document_ids.iter().map(ToString::to_string).collect();
    vec![
        button("Mark as done", DashboardAction::SettleRetention { document_ids: ids.clone() })
            .icon("check")
            .style(DashboardButtonStyle::Tonal),
        button("Move to trash…", DashboardAction::TrashRetained { document_ids: ids })
            .icon("delete")
            .confirm(trash_retained(document_ids, context)),
    ]
}

/// The files `task` is about, the primary's first; one per folder when their versions differ.
pub fn files_for(task: &OpenTask, context: &TaskContext<'_>) -> Vec<TaskFile> {
    match task {
        OpenTask::ReviewSuggestion(suggestion) => document_file(&suggestion.document_id, context),
        OpenTask::DeadlineDue { document_id, .. }
        | OpenTask::RetentionDue { document_id, .. }
        | OpenTask::BillOverdue { document_id, .. }
        | OpenTask::UnreadableDocument { document_id } => document_file(document_id, context),
        OpenTask::DamagedCopy { document_id, folder_id, .. } => context
            .path_of(document_id)
            .map(|path| task_file(&format!("Open {} copy", context.folder_name(folder_id)), folder_id, path))
            .into_iter()
            .collect(),
        OpenTask::AdoptNewFile(change) => first_of(&change.present_in, &change.path, context),
        OpenTask::RestoreOrDelete(change) => first_of(&change.present_in, &change.path, context),
        OpenTask::ReviewModification(change) => {
            let changed = Some(task_file("Open new version", &change.folder_id, &change.path));
            let previous = context
                .other_folder_than(&change.folder_id)
                .map(|folder| task_file("Open previous version", folder, &change.path));
            changed.into_iter().chain(previous).collect()
        }
        OpenTask::ConfirmMove(change) => vec![task_file("Open", &change.folder_id, &change.path)],
        OpenTask::ResolveDuplicates { document_ids, .. } => {
            let Some(primary) = context.primary_id() else { return Vec::new() };
            context
                .documents(document_ids)
                .into_iter()
                .map(|document| task_file(&format!("Open {}", document.path.file_name()), primary, &document.path))
                .collect()
        }
        OpenTask::ResolveContentConflict(change) => context
            .primary_first(change.content_hashes.keys())
            .into_iter()
            .map(|folder| task_file(&format!("Open {} version", context.folder_name(folder)), folder, &change.path))
            .collect(),
        OpenTask::ReviewCatalogMerge(_)
        | OpenTask::RetryFailedOperations { .. }
        | OpenTask::RestructureSuggested { .. }
        | OpenTask::ReviewLearnedRule(_)
        | OpenTask::ExpectedDocumentMissing(_)
        | OpenTask::ReviewContractProposal(_)
        | OpenTask::ContractCancelBy { .. }
        | OpenTask::FollowUpDue { .. }
        | OpenTask::CaseWaitingTooLong { .. }
        | OpenTask::EmergencyFolderRefresh { .. } => Vec::new(),
    }
}

/// A text button for each of `files` that opens it in its default application; it never waits.
pub fn opens_of(files: &[TaskFile]) -> Vec<DashboardButtonDto> {
    files.iter().map(open_button).collect()
}

fn open_button(file: &TaskFile) -> DashboardButtonDto {
    let action = DashboardAction::Open { folder_id: file.file.folder_id.clone(), path: file.file.path.clone() };
    button(file.label.clone(), action).icon("open_in_new").style(DashboardButtonStyle::Text)
}

fn task_file(label: &str, folder_id: &LibraryFolderId, path: &RelativePath) -> TaskFile {
    TaskFile { label: label.to_owned(), file: TaskFileDto { folder_id: folder_id.to_string(), path: path.to_string() } }
}

fn document_file(document_id: &DocumentId, context: &TaskContext<'_>) -> Vec<TaskFile> {
    match (context.path_of(document_id), context.primary_id()) {
        (Some(path), Some(primary)) => vec![task_file("Open", primary, path)],
        _ => Vec::new(),
    }
}

fn first_of<'i>(
    folders: impl IntoIterator<Item = &'i LibraryFolderId>,
    path: &RelativePath,
    context: &TaskContext<'_>,
) -> Vec<TaskFile> {
    context.primary_first(folders).into_iter().take(1).map(|folder| task_file("Open", folder, path)).collect()
}

fn new_file_buttons(key: &str, change: &NewFile, context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let task_key = key.to_owned();
    if let Some(original) = context.original_of(change) {
        return vec![
            button("Move copy to trash…", DashboardAction::TrashDuplicateFile { task_key: task_key.clone() })
                .icon("delete")
                .style(DashboardButtonStyle::Tonal)
                .confirm(trash_duplicate_file(change, original, context)),
            button("Adopt anyway", DashboardAction::Adopt { task_key }),
        ];
    }
    vec![
        button("Adopt", DashboardAction::Adopt { task_key: task_key.clone() })
            .icon("check")
            .style(DashboardButtonStyle::Tonal)
            .disabled(change.content_hash.is_none().then_some("The file was not read yet; scan again")),
        button("Move to trash…", DashboardAction::TrashNewFile { task_key }).confirm(trash_new_file(change, context)),
    ]
}

fn modification_buttons(key: &str, change: &ModifiedFile, context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let path = &change.path;
    let keep_new = button(
        "Keep new content…",
        DashboardAction::KeepVersion { task_key: key.to_owned(), folder_id: change.folder_id.to_string() },
    )
    .confirm(keep_version(path, &change.folder_id, context));
    let previous = context.other_folder_than(&change.folder_id);
    let restore = match previous {
        Some(folder) => button(
            "Restore previous…",
            DashboardAction::KeepVersion { task_key: key.to_owned(), folder_id: folder.to_string() },
        )
        .confirm(keep_version(path, folder, context)),
        // Disabled; an empty folder id is refused should it ever be sent.
        None => button(
            "Restore previous…",
            DashboardAction::KeepVersion { task_key: key.to_owned(), folder_id: String::new() },
        )
        .disabled(Some("No other folder has the previous content")),
    };
    vec![keep_new, restore]
}

fn conflict_buttons(key: &str, change: &ContentConflict, context: &TaskContext<'_>) -> Vec<DashboardButtonDto> {
    let folders = context.primary_first(change.content_hashes.keys());
    let primary = context.primary_id();
    let mut buttons: Vec<DashboardButtonDto> = folders
        .iter()
        .map(|folder| {
            button(
                format!("Keep {}…", context.folder_name(folder)),
                DashboardAction::KeepVersion { task_key: key.to_owned(), folder_id: folder.to_string() },
            )
            .confirm(keep_version(&change.path, folder, context))
        })
        .collect();
    let renamed = folders.iter().find(|folder| Some(**folder) != primary);
    buttons.push(match renamed {
        Some(folder) => {
            button("Keep both", DashboardAction::KeepBoth { task_key: key.to_owned(), folder_id: folder.to_string() })
        }
        None => button("Keep both", DashboardAction::KeepBoth { task_key: key.to_owned(), folder_id: String::new() })
            .disabled(Some("Only the primary folder has a version")),
    });
    buttons
}

/// The copies of a duplicates task, the recommended one first; empty for other tasks or fewer than two copies.
pub fn copies_for(task: &OpenTask, context: &TaskContext<'_>) -> Vec<DuplicateCopyDto> {
    let OpenTask::ResolveDuplicates { document_ids, .. } = task else { return Vec::new() };
    let copies = context.documents(document_ids);
    let Some(primary) = context.primary_id() else { return Vec::new() };
    if copies.len() < 2 {
        return Vec::new();
    }
    let owned: Vec<Document> = copies.iter().map(|copy| (*copy).clone()).collect();
    let Some(preferred) = DuplicatePreference::preferred(&owned).map(|document| document.id.clone()) else {
        return Vec::new();
    };
    let mut sorted = copies;
    sorted.sort_by_key(|copy| copy.id != preferred);
    let key = task.key().value().to_owned();
    sorted
        .into_iter()
        .map(|copy| {
            let recommended = copy.id == preferred;
            let values = assigned_values(copy, context);
            let keep = button(
                "Keep…",
                DashboardAction::KeepDuplicate { task_key: key.clone(), document_id: copy.id.to_string() },
            )
            .style(if recommended { DashboardButtonStyle::Tonal } else { DashboardButtonStyle::Outlined })
            .waits_for(Lane::Library)
            .confirm(keep_duplicate(copy, document_ids, context));
            DuplicateCopyDto {
                path: copy.path.to_string(),
                detail: if recommended { format!("Recommended · {values}") } else { values },
                open: open_button(&task_file("Open", primary, &copy.path)),
                keep,
            }
        })
        .collect()
}

fn assigned_values(document: &Document, context: &TaskContext<'_>) -> String {
    let mut values = Vec::new();
    if let Some(category) = &document.category_id {
        values.push(format!("Category: {}", context.category_name(category)));
    }
    if !document.tags.is_empty() {
        values.push(format!("Tags: {}", tag_list(document.tags.iter().map(dms_domain::taxonomy::TagName::value))));
    }
    if let Some(title) = &document.metadata.title {
        values.push(format!("Title: {title}"));
    }
    if values.is_empty() { "Nothing assigned".to_owned() } else { values.join(" · ") }
}

pub fn tag_list<'t>(tags: impl Iterator<Item = &'t str>) -> String {
    tags.collect::<Vec<_>>().join(", ")
}

fn affected_in<'i>(
    folders: impl IntoIterator<Item = &'i LibraryFolderId>,
    path: &RelativePath,
    context: &TaskContext<'_>,
) -> Vec<String> {
    let wanted: Vec<&LibraryFolderId> = folders.into_iter().collect();
    context
        .folders()
        .iter()
        .filter(|folder| wanted.contains(&&folder.id))
        .map(|folder| format!("{}: {path}", folder.display_name))
        .collect()
}

fn affected_everywhere(path: &RelativePath, context: &TaskContext<'_>) -> Vec<String> {
    context.folders().iter().map(|folder| format!("{}: {path}", folder.display_name)).collect()
}

fn trash_new_file(change: &NewFile, context: &TaskContext<'_>) -> DashboardConfirmationDto {
    confirmation(
        "Move to trash".to_owned(),
        "The file is not in the library. It goes to the trash in the folders that have it.".to_owned(),
        affected_in(&change.present_in, &change.path, context),
        "Move to trash",
    )
}

fn trash_duplicate_file(change: &NewFile, original: &Document, context: &TaskContext<'_>) -> DashboardConfirmationDto {
    confirmation(
        "Move copy to trash".to_owned(),
        format!(
            "The file has exactly the same size and SHA-256 checksum as {}. Right before it goes to the trash it is \
             compared with it byte by byte once more, in each folder.",
            original.path
        ),
        affected_in(&change.present_in, &change.path, context),
        "Move to trash",
    )
}

fn delete_everywhere(change: &MissingFile, context: &TaskContext<'_>) -> DashboardConfirmationDto {
    confirmation(
        "Delete everywhere".to_owned(),
        "The file goes to the trash in every library folder and leaves the catalog.".to_owned(),
        affected_everywhere(&change.path, context),
        "Move to trash",
    )
}

fn keep_version(path: &RelativePath, source: &LibraryFolderId, context: &TaskContext<'_>) -> DashboardConfirmationDto {
    let name = context.folder_name(source);
    let others: Vec<String> = context
        .folders()
        .iter()
        .filter(|folder| &folder.id != source)
        .map(|folder| format!("{}: {path}", folder.display_name))
        .collect();
    confirmation(
        format!("Keep the version of {name}"),
        format!("Every other version of this file goes to the trash and is replaced by the one in {name}."),
        others,
        "Keep this version",
    )
}

fn keep_duplicate(kept: &Document, document_ids: &[DocumentId], context: &TaskContext<'_>) -> DashboardConfirmationDto {
    let copies = context.documents(document_ids).into_iter().filter(|copy| copy.id != kept.id);
    let affected = copies.flat_map(|copy| affected_everywhere(&copy.path, context)).collect();
    confirmation(
        format!("Keep {}", kept.path),
        format!(
            "The other copies have exactly the same size and SHA-256 checksum. Right before they go to the trash in \
             every folder, each is compared byte by byte once more. Their categories, tags and title move to {} \
             where it has none.",
            kept.path.file_name()
        ),
        affected,
        "Move copies to trash",
    )
}

fn repair_copy(
    document_id: &DocumentId,
    damaged: &LibraryFolderId,
    source: &LibraryFolderId,
    context: &TaskContext<'_>,
) -> DashboardConfirmationDto {
    let path = context.document_path(document_id);
    confirmation(
        format!("Repair from {}", context.folder_name(source)),
        format!(
            "The damaged copy goes to the trash of {} and the intact copy of {} takes its place. The intact copy is \
             compared with the catalog once more right before.",
            context.folder_name(damaged),
            context.folder_name(source)
        ),
        vec![format!("{}: {path}", context.folder_name(damaged))],
        "Repair",
    )
}

pub fn trash_retained(document_ids: &[DocumentId], context: &TaskContext<'_>) -> DashboardConfirmationDto {
    let paths: Vec<&RelativePath> = document_ids.iter().filter_map(|id| context.path_of(id)).collect();
    confirmation(
        format!("Move {} documents to the trash", paths.len()),
        "You chose to keep them until a date that has passed. They go to the trash in every library folder and \
         leave the catalog."
            .to_owned(),
        paths.iter().flat_map(|path| affected_everywhere(path, context)).collect(),
        "Move to trash",
    )
}

/// The question before removing every duplicate of the group at once (Kotlin `BulkActions.removeAll`).
pub fn remove_all_duplicates(
    files: &[&NewFile],
    catalogued: &[&[DocumentId]],
    context: &TaskContext<'_>,
) -> DashboardConfirmationDto {
    let mut affected: Vec<String> = files
        .iter()
        .map(|file| match context.original_of(file) {
            Some(original) => format!("{} (copy of {})", file.path, original.path),
            None => format!("{} (copy of a document)", file.path),
        })
        .collect();
    for document_ids in catalogued {
        let copies: Vec<Document> = context.documents(document_ids).into_iter().cloned().collect();
        if let Some(kept) = DuplicatePreference::preferred(&copies) {
            affected.extend(
                copies
                    .iter()
                    .filter(|copy| copy.id != kept.id)
                    .map(|copy| format!("{} (copy of {})", copy.path, kept.path)),
            );
        }
    }
    confirmation(
        "Remove all duplicates".to_owned(),
        "Each file has exactly the same size and SHA-256 checksum as the document named after it, and is compared \
         with it byte by byte once more right before it goes to the trash, in every folder. Where a document keeps \
         one of its copies, it takes over the copy's categories and tags."
            .to_owned(),
        affected,
        "Move duplicates to trash",
    )
}
