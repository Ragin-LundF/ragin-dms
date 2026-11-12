//! The words for every outcome a use case can return (Kotlin `OutcomeMessages`). Use cases return data; only the
//! UI layer phrases it, here, so every command reports the same outcome the same way.

use dms_application::classification::SuggestOutcome;
use dms_application::common::OperationOutcome;
use dms_application::contract::ExtractContractTermsOutcome;
use dms_application::health::VerifyOutcome;
use dms_application::letter::{DraftLetterOutcome, DraftSource, SaveLetterOutcome};
use dms_application::port::ai::ClassificationFailure;
use dms_application::port::storage::StorageFailure;
use dms_application::search::TranslateOutcome;
use dms_application::split::{ApplySplitOutcome, ProposeSplitOutcome};
use dms_application::sync::{RetryOutcome, SyncOutcome};
use dms_domain::bill::EpcRefusal;
use dms_domain::catalog::CatalogRejection;
use dms_domain::library::FileNameViolation;
use dms_domain::split::SplitProblem;

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

/// The message after an archive check; `None` when it was cancelled, which the session already reports.
pub fn verify(outcome: &VerifyOutcome) -> Option<UiMessage> {
    match outcome {
        VerifyOutcome::Finished(report) => {
            let skipped: u32 = report.folders.iter().map(|folder| folder.not_downloaded).sum();
            let skipped =
                if skipped > 0 { format!("; {skipped} files not downloaded, skipped") } else { String::new() };
            Some(match report.findings.len() {
                0 => UiMessage::info(format!("Archive checked: every copy is intact{skipped}")),
                1 => UiMessage::error(format!("Archive checked: 1 problem found{skipped}")),
                count => UiMessage::error(format!("Archive checked: {count} problems found{skipped}")),
            })
        }
        VerifyOutcome::Cancelled(_) => None,
        VerifyOutcome::LibraryNotOpen => Some(UiMessage::error("No library is open")),
        VerifyOutcome::StorageFailed(error) => {
            Some(UiMessage::error(format!("The check results could not be saved ({})", failure(error.reason))))
        }
    }
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
        CatalogRejection::ContractNotFound => "The contract no longer exists",
        CatalogRejection::CaseNotFound => "The case no longer exists",
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

/// The hint under a drafted letter: who wrote it, and why the template when the AI failed.
pub fn letter_origin(source: &DraftSource) -> String {
    let failure = match source {
        DraftSource::Ai { model } => return format!("Drafted by AI ({model})"),
        DraftSource::Template { ai_failure: None } => return "Template".to_owned(),
        DraftSource::Template { ai_failure: Some(reason) } => reason,
    };
    let why = match failure {
        ClassificationFailure::ProviderUnavailable => "the AI does not answer",
        ClassificationFailure::ModelCannotReadImages => "the model cannot read the documents",
        ClassificationFailure::InvalidResponse => "the AI's answer could not be understood",
        ClassificationFailure::Timeout => "the AI took too long",
    };
    format!("Template: {why}")
}

/// Why no letter was drafted; `None` for a draft.
pub fn letter_problem(outcome: &DraftLetterOutcome) -> Option<&'static str> {
    match outcome {
        DraftLetterOutcome::Drafted { .. } => None,
        DraftLetterOutcome::LibraryNotOpen => Some("No library is open"),
        DraftLetterOutcome::NotFound => Some("The document or file no longer exists"),
    }
}

/// Why the AI read no contract terms; `None` for a reading.
pub fn contract_terms_problem(outcome: &ExtractContractTermsOutcome) -> Option<String> {
    let why = match outcome {
        ExtractContractTermsOutcome::Suggested(_) => return None,
        ExtractContractTermsOutcome::LibraryNotOpen => "No library is open",
        ExtractContractTermsOutcome::NotFound => rejection(CatalogRejection::ContractNotFound),
        ExtractContractTermsOutcome::AiDisabled => "AI is off on this device",
        ExtractContractTermsOutcome::NoText => "None of the contract's documents has text the AI can read",
        ExtractContractTermsOutcome::Failed(reason) => match reason {
            ClassificationFailure::ProviderUnavailable => "The AI does not answer",
            ClassificationFailure::ModelCannotReadImages => "The model cannot read the documents",
            ClassificationFailure::InvalidResponse => "The AI's answer could not be understood",
            ClassificationFailure::Timeout => "The AI took too long",
        },
    };
    Some(why.to_owned())
}

pub fn letter_saved(outcome: &SaveLetterOutcome) -> UiMessage {
    match outcome {
        SaveLetterOutcome::Saved { path, pending_folders, .. } => {
            done_with_pending(&format!("Saved the letter as {}", path.file_name()), pending_folders.len())
        }
        SaveLetterOutcome::Refused(refusal) => of(refusal, "Saved the letter"),
    }
}

/// Why there is no split proposal; `None` when there is one.
pub fn split_problem(outcome: &ProposeSplitOutcome) -> Option<String> {
    let text = match outcome {
        ProposeSplitOutcome::Proposed { .. } => return None,
        ProposeSplitOutcome::LibraryNotOpen => "No library is open",
        ProposeSplitOutcome::NotFound => "The document or file no longer exists",
        ProposeSplitOutcome::NotPdf => "Only PDFs can be split",
        ProposeSplitOutcome::TooFewPages => "A stack needs at least two pages",
        ProposeSplitOutcome::Cancelled => "Stopped reading the pages",
        ProposeSplitOutcome::Unreadable(error) => failure(error.reason),
        ProposeSplitOutcome::Failed { reason, .. } => crate::features::document::view::texts::failure(*reason),
    };
    Some(text.to_owned())
}

/// Why the AI's parts were not used and the stack shows as one part.
pub fn split_fallback(problem: SplitProblem) -> &'static str {
    match problem {
        SplitProblem::NoParts => "The AI found no documents in this stack; set the cuts yourself",
        SplitProblem::Gap => "The AI's parts left out pages; set the cuts yourself",
        SplitProblem::Overlap => "The AI's parts overlapped; set the cuts yourself",
        SplitProblem::BeyondLastPage => "The AI's parts went past the last page; set the cuts yourself",
    }
}

/// What "Split and review" did with the stack `name`.
pub fn split_applied(outcome: &ApplySplitOutcome, name: &str) -> UiMessage {
    match outcome {
        ApplySplitOutcome::Split { outcome, parts, .. } => {
            of(outcome, &format!("Split {name} into {} documents; each one waits for your review", parts.len()))
        }
        ApplySplitOutcome::NotSplit(outcome) => of(outcome, ""),
        ApplySplitOutcome::InvalidParts(_) => UiMessage::error("The parts must cover every page in order"),
        ApplySplitOutcome::NothingToSplit => UiMessage::error("Mark at least one cut to split the stack"),
    }
}

/// Why a sentence did not become filters; `None` when it did.
pub fn translate_problem(outcome: &TranslateOutcome) -> Option<&'static str> {
    match outcome {
        TranslateOutcome::Translated(_) => None,
        TranslateOutcome::LibraryNotOpen => Some("No library is open"),
        TranslateOutcome::BlankSentence => Some("Type what you are looking for first"),
        TranslateOutcome::Failed { reason, .. } => Some(crate::features::document::view::texts::failure(*reason)),
    }
}

fn done_with_pending(text: &str, pending_folders: usize) -> UiMessage {
    if pending_folders == 0 {
        return UiMessage::info(text);
    }
    UiMessage::info(format!("{text}; {pending_folders} sync folder(s) get the change when retried"))
}

/// Why a bill gets no payment code (`GiroCode`); `None` for a bill without an IBAN.
pub fn payment_code_refusal(refusal: Option<&EpcRefusal>) -> String {
    match refusal {
        None => "The bill has no valid IBAN; add it in the details".to_owned(),
        Some(EpcRefusal::NotEuro(currency)) => format!("Payment codes only work for euro amounts, not {currency}"),
        Some(EpcRefusal::AmountOutOfRange) => "The amount is outside what a payment code can carry".to_owned(),
        Some(EpcRefusal::PayeeMissing) => "The bill names no payee; add a correspondent".to_owned(),
        Some(EpcRefusal::PayeeTooLong) => "The payee's name is longer than 70 characters".to_owned(),
        Some(EpcRefusal::ReferenceTooLong) => "The payment reference is longer than 140 characters".to_owned(),
    }
}

/// The message after asking for a suggestion from a PDF's e-invoice.
pub fn e_invoice_suggestion(outcome: &SuggestOutcome, name: &str) -> UiMessage {
    match outcome {
        SuggestOutcome::Suggested(_) => UiMessage::info(format!("Read the e-invoice of {name}; review its values")),
        SuggestOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        SuggestOutcome::NotFound => UiMessage::error(format!("{name} is no longer in the library")),
        SuggestOutcome::NotReadable { .. } => UiMessage::error(format!("{name} holds no e-invoice")),
        SuggestOutcome::Failed { .. } => UiMessage::error(format!("The e-invoice of {name} could not be read")),
    }
}
/// What became of the emergency overview or the encrypted ZIP; `done` names the action.
pub fn emergency_outcome(outcome: &dms_application::emergency::EmergencyOutcome, done: &str) -> UiMessage {
    use dms_application::emergency::EmergencyOutcome;
    match outcome {
        EmergencyOutcome::Saved { location, .. } => UiMessage::info(format!("{done}: {location}")),
        EmergencyOutcome::InsideLibrary => {
            UiMessage::error("Choose a place outside the library, or save the overview into a library folder")
        }
        EmergencyOutcome::NothingIncluded => UiMessage::error("Switch on at least one item of the emergency folder"),
        EmergencyOutcome::Refused(refusal) => of(refusal, done),
    }
}

/// One line for an email import: how many mails became documents and which failed, with the reason of the first.
pub fn emails_imported(outcome: &dms_application::mail::ImportEmailsOutcome) -> UiMessage {
    use dms_application::mail::ImportEmailsOutcome;
    let imported = match outcome {
        ImportEmailsOutcome::LibraryNotOpen => return UiMessage::error("No library is open"),
        ImportEmailsOutcome::Imported(imported) => imported,
    };
    let done = imported.iter().filter(|email| matches!(email.outcome, OperationOutcome::Done { .. })).count();
    let attachments: usize = imported.iter().map(|email| email.attachments.len()).sum();
    let Some(failed) = imported.iter().find(|email| !matches!(email.outcome, OperationOutcome::Done { .. })) else {
        return UiMessage::info(format!("Imported {done} email(s) with {attachments} attachment(s)"));
    };
    let reason = of(&failed.outcome, "").text;
    let failures = imported.len() - done;
    UiMessage::error(format!("Imported {done} email(s); {failures} failed, {}: {reason}", failed.source))
}
