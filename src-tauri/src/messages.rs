//! The words for every outcome a use case can return (Kotlin `OutcomeMessages`). Use cases return data; only the
//! UI layer phrases it, here, so every command reports the same outcome the same way.

use dms_application::common::OperationOutcome;
use dms_application::port::storage::StorageFailure;
use dms_application::sync::{RetryOutcome, SyncOutcome};
use dms_domain::catalog::CatalogRejection;
use dms_domain::library::FileNameViolation;

use crate::session::UiMessage;

/// The message for a change's outcome; `done` is the text for success.
pub fn of(outcome: &OperationOutcome, done: &str) -> UiMessage {
    match outcome {
        OperationOutcome::Done { pending_folders, .. } => done_with_pending(done, pending_folders.len()),
        OperationOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        OperationOutcome::NotFound => UiMessage::error("The document or file no longer exists"),
        OperationOutcome::ScanOutdated => UiMessage::error("The folders changed since the last scan; scan again"),
        OperationOutcome::SourceChanged => UiMessage::error("The file changed in the meantime; nothing was changed"),
        OperationOutcome::NotConfirmed => UiMessage::error("Nothing was changed without confirmation"),
        OperationOutcome::InvalidName(violation) => UiMessage::error(name_problem(*violation)),
        OperationOutcome::NameTaken => UiMessage::error("Another entry there already has this name (ignoring case)"),
        OperationOutcome::Rejected(reason) => UiMessage::error(rejection(*reason)),
        OperationOutcome::StorageFailed(error) => UiMessage::error(format!(
            "The primary folder could not be changed ({}); nothing was changed",
            failure(error.reason)
        )),
        // New in Rust: Kotlin threw IllegalArgumentException here; its message names the broken rule.
        OperationOutcome::InvalidInput(error) => UiMessage::error(error.to_string()),
    }
}

/// The message of a sync or merge: what was copied or moved, and what still needs attention.
pub fn sync(outcome: &SyncOutcome, done: &str) -> UiMessage {
    match outcome {
        SyncOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        SyncOutcome::PrimaryFailed(error) => {
            UiMessage::error(format!("The primary folder could not be read ({})", failure(error.reason)))
        }
        SyncOutcome::Completed(report) => {
            let mut parts = vec![format!("{} file(s) copied or moved", report.actions.len())];
            let counts = [
                (report.conflicts.len(), "conflict(s) to resolve"),
                (report.failed_paths.len(), "file(s) failed"),
                (report.unreachable_folders.len(), "folder(s) offline"),
            ];
            parts.extend(counts.iter().filter(|(count, _)| *count > 0).map(|(count, text)| format!("{count} {text}")));
            UiMessage::info(format!("{done}: {}", parts.join(", ")))
        }
    }
}

pub fn retry(outcome: &RetryOutcome) -> UiMessage {
    let RetryOutcome::Retried(report) = outcome else { return UiMessage::error("No library is open") };
    let text = format!(
        "Retried: {} done, {} no longer needed, {} still pending",
        report.succeeded, report.dropped, report.still_pending
    );
    UiMessage { text, is_error: report.still_pending > 0 }
}

/// Why a button waits: the `operation` running in its lane.
pub fn wait_for(operation: &str) -> String {
    format!("Wait until \"{operation}\" finishes")
}

pub fn failure(reason: StorageFailure) -> &'static str {
    match reason {
        StorageFailure::FolderUnreachable => "folder not reachable",
        StorageFailure::AlreadyExists => "the target already exists",
        StorageFailure::NotFound => "file not found",
        StorageFailure::VerificationFailed => "the copy did not match the original",
        StorageFailure::IoError => "read or write error",
        StorageFailure::UnsupportedFormat => "written by a newer version or damaged",
        StorageFailure::CatalogChanged => "another device changed the catalog at the same time; try again",
        StorageFailure::Cancelled => "cancelled",
    }
}

pub fn rejection(reason: CatalogRejection) -> &'static str {
    match reason {
        CatalogRejection::DocumentAlreadyExists => "The document is already in the catalog",
        CatalogRejection::DocumentNotFound => "The document is no longer in the catalog",
        CatalogRejection::PathAlreadyUsed => "Another document already uses this path",
        CatalogRejection::InvalidFolderMove => "A folder cannot be moved into itself",
        CatalogRejection::CategoryAlreadyExists => "The category already exists",
        CatalogRejection::CategoryNotFound => "The category no longer exists",
        CatalogRejection::CategoryNameTaken => "A category with this name already exists there",
        CatalogRejection::CategoryInUse => "Documents still use this category",
        CatalogRejection::CategoryHasChildren => "The category still has subcategories",
        CatalogRejection::CategoryCycle => "A category cannot be placed below itself",
        CatalogRejection::TagNotFound => "The tag no longer exists",
        CatalogRejection::FieldNotDefined => "The library has no such field",
        CatalogRejection::FieldTypeMismatch => "The value does not fit the field's type",
        CatalogRejection::LinkNotFound => "The documents are not linked",
    }
}

pub fn name_problem(violation: FileNameViolation) -> &'static str {
    match violation {
        FileNameViolation::Empty => "The name is empty",
        FileNameViolation::TooLong => "The name is too long",
        FileNameViolation::ControlCharacter => "The name contains control characters",
        FileNameViolation::ForbiddenCharacter => "The name contains one of < > : \" / \\ | ? *",
        FileNameViolation::ReservedName => "Windows reserves this name",
        FileNameViolation::LeadingDotOrSpace => "The name must not start with a dot or a space",
        FileNameViolation::TrailingDotOrSpace => "The name must not end with a dot or a space",
    }
}

fn done_with_pending(text: &str, pending_folders: usize) -> UiMessage {
    if pending_folders == 0 {
        return UiMessage::info(text);
    }
    UiMessage::info(format!("{text}; {pending_folders} sync folder(s) get the change when retried"))
}
