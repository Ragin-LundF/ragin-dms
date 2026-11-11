//! The document area's reach into the session and the services: lookups shared by the commands, and one function
//! per use case call, run as the session operation Kotlin's view models used.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::NaiveDate;
use dms_application::ask::RelatedOutcome;
use dms_application::assistant::AnswerOutcome;
use dms_application::classification::{SuggestOutcome, SuggestionApproval};
use dms_application::common::OperationOutcome;
use dms_application::port::assistant::ChatTurn;
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::RelativePath;
use dms_domain::rule::MatchingRuleId;
use dms_domain::settings::CatalogSettings;
use dms_domain::task::OpenTask;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::document::dto::OpenReviewEvent;
use crate::features::document::view::edits::DocumentEdit;
use crate::features::document::view::texts;
use crate::messages;
use crate::session::reporting::busy;
use crate::session::{Lane, Operation, SessionFolder, UiMessage};

/// How many related documents the details show (Kotlin `RelatedDocuments.DEFAULT_LIMIT`).
const RELATED_LIMIT: usize = 5;

pub fn document_id(raw: &str) -> Result<DocumentId, IpcError> {
    DocumentId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))
}

/// The catalogued document `raw`, or `invalid_input` when it left the catalog.
pub fn find_document<'a>(catalog: &'a Catalog, raw: &str) -> Result<&'a Document, IpcError> {
    catalog
        .documents()
        .get(&document_id(raw)?)
        .ok_or_else(|| IpcError::invalid("The document is no longer in the catalog"))
}

pub fn primary_folder(ctx: &AppContext) -> Option<SessionFolder> {
    ctx.session.state().folders.into_iter().find(|folder| folder.primary)
}

/// The file of `document` in the primary folder, where every document lives; `None` without a primary folder.
pub fn primary_file(ctx: &AppContext, document: &Document) -> Option<PathBuf> {
    let primary = primary_folder(ctx)?;
    Some(document.path.segments().into_iter().fold(PathBuf::from(primary.location), |path, segment| path.join(segment)))
}

/// Runs `job` with the services as the one operation of `lane`; `busy` names the running operation.
fn perform<F>(ctx: &AppContext, label: &str, lane: Lane, cancellable: bool, job: F) -> Result<(), IpcError>
where
    F: FnOnce(&AppServices, &Operation) -> Result<(), String> + Send + 'static,
{
    let services = Arc::clone(ctx.require_services()?);
    if ctx.session.perform(label, lane, cancellable, move |operation| job(&services, operation)) {
        Ok(())
    } else {
        Err(busy(&ctx.session, lane, label))
    }
}

pub fn report_error(ctx: &AppContext, text: impl Into<String>) {
    ctx.session.report(UiMessage::error(text));
}

/// Today in the library's time zone.
pub fn today(ctx: &AppContext) -> NaiveDate {
    ctx.services().map_or_else(|| chrono::Local::now().date_naive(), |services| services.library.clock.today())
}

/// The device setting "Rename to the suggested file name".
pub fn accept_file_names(ctx: &AppContext) -> bool {
    ctx.services()
        .and_then(|services| services.library.settings.load_preferences().ok())
        .is_some_and(|preferences| preferences.ai.accept_suggested_file_names)
}

/// The suggestion that waits for review of `document_id`, from the session's dashboard (Kotlin `DashboardFeed`).
pub fn pending_suggestion(ctx: &AppContext, document_id: &DocumentId) -> Option<ClassificationSuggestion> {
    ctx.dashboard()?.tasks.into_iter().find_map(|task| match task {
        OpenTask::ReviewSuggestion(suggestion) if &suggestion.document_id == document_id => Some(suggestion),
        _ => None,
    })
}

/// The display names of the configured library folders, for the trash question.
pub fn folder_names(ctx: &AppContext) -> Result<Vec<String>, IpcError> {
    let folders = ctx
        .require_services()?
        .library
        .settings
        .load_library_folders()
        .map_err(|error| IpcError::internal(messages::failure(error.reason)))?;
    Ok(folders
        .map(|folders| folders.folders().iter().map(|folder| folder.display_name.clone()).collect())
        .unwrap_or_default())
}

/// One catalog change as "Saving the catalog"; only failures are reported.
pub fn save_edit(ctx: &AppContext, edit: &DocumentEdit) -> Result<(), IpcError> {
    let edit = edit.clone();
    perform(ctx, "Saving the catalog", Lane::Library, false, move |services, operation| {
        let outcome = match &edit {
            DocumentEdit::Catalog(edit) => services.documents.edit_catalog.run(edit),
            DocumentEdit::Details(edit) => services.documents.edit_document_details.run(edit),
        };
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            operation.session().report(messages::of(&outcome, ""));
        }
        Ok(())
    })
}

/// Changes the library settings as `label`, reporting `done` (or the failure).
pub fn change_settings(
    ctx: &AppContext,
    label: &str,
    subject: &str,
    done: &str,
    change: impl Fn(&CatalogSettings) -> CatalogSettings + Send + 'static,
) -> Result<(), IpcError> {
    let (subject, done) = (subject.to_owned(), done.to_owned());
    perform(ctx, label, Lane::Library, false, move |services, operation| {
        let outcome = services.documents.change_settings.run(&subject, change);
        operation.session().report(messages::of(&outcome, &done));
        Ok(())
    })
}

/// A new id for a rule (Kotlin `UUID.randomUUID()`).
pub fn new_rule_id() -> MatchingRuleId {
    MatchingRuleId::new(uuid::Uuid::new_v4().to_string()).expect("invariant: a UUID is not blank")
}

/// Moves `document` to `target` in every library folder as "Moving {path}"; reports "Moved to {target}".
pub fn move_document(ctx: &AppContext, document: &Document, target: &RelativePath) -> Result<(), IpcError> {
    let (id, target) = (document.id.clone(), target.clone());
    perform(ctx, &format!("Moving {}", document.path.value()), Lane::Library, false, move |services, operation| {
        let outcome = services.documents.move_document.run(&id, &target);
        operation.session().report(messages::of(&outcome, &format!("Moved to {}", target.value())));
        Ok(())
    })
}

/// Trashes `document` in every library folder as "Moving {path} to the trash".
pub fn trash_document(ctx: &AppContext, document: &Document) -> Result<(), IpcError> {
    let (id, path) = (document.id.clone(), document.path.value().to_owned());
    perform(ctx, &format!("Moving {path} to the trash"), Lane::Library, false, move |services, operation| {
        let outcome = services.documents.trash_document.run(&id, true);
        operation.session().report(messages::of(&outcome, &format!("Moved {path} to the trash")));
        Ok(())
    })
}

/// What the AI suggests for one document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRun {
    Classify,
    Tag,
}

/// Lets the AI suggest a classification or tags in the AI lane, cancellable, labelled `label`; a suggestion opens
/// its review in the window (`OpenReviewEvent`), tags only when there are new ones.
pub fn suggest_with_ai(
    ctx: &AppContext,
    app: AppHandle,
    document: &Document,
    run: AiRun,
    label: &str,
) -> Result<(), IpcError> {
    let (id, name) = (document.id.clone(), document.path.file_name().to_owned());
    perform(ctx, label, Lane::Ai, true, move |services, operation| {
        let outcome = match run {
            AiRun::Classify => services.ai.suggest_classification.run(&id),
            AiRun::Tag => services.ai.suggest_tags.run(&id),
        };
        let session = operation.session();
        match &outcome {
            SuggestOutcome::Suggested(suggestion) if run == AiRun::Tag && suggestion.tags.is_empty() => {
                session.report(UiMessage::info(format!("The AI found no new tags for {name}")));
            }
            SuggestOutcome::Suggested(_) => {
                // A new revision first, so the review reads a dashboard that has the suggestion.
                session.changed();
                let _ = OpenReviewEvent { document_id: id.value().to_owned() }.emit(&app);
            }
            _ => session.report(texts::suggest_message(&outcome, &name)),
        }
        Ok(())
    })
}

/// Applies `approval` as "Applying the suggestion for {path}", reports "Filed {path}", then embeds the metadata into
/// the PDF when the library wants that.
pub fn approve(ctx: &AppContext, document: &Document, approval: SuggestionApproval) -> Result<(), IpcError> {
    let (id, path) = (document.id.clone(), document.path.value().to_owned());
    perform(ctx, &format!("Applying the suggestion for {path}"), Lane::Library, false, move |services, operation| {
        let session = operation.session();
        let outcome = services.ai.approve_suggestion.run(&id, &approval);
        session.report(messages::of(&outcome, &format!("Filed {path}")));
        match &outcome {
            OperationOutcome::Done { catalog, .. } if catalog.settings().layout.embed_metadata => {
                let embedded = services.pdf.embed_pdf_metadata.run(&id);
                if !matches!(embedded, OperationOutcome::Done { .. }) {
                    session.report(messages::of(&embedded, ""));
                }
            }
            OperationOutcome::Done { .. } => {}
            _ => session.error(format!("Approval refused for {path}")),
        }
        Ok(())
    })
}

/// Removes the suggestion for `document` as `label`; nothing else changes.
pub fn reject(ctx: &AppContext, document: &Document, label: &str) -> Result<(), IpcError> {
    let id = document.id.clone();
    perform(ctx, label, Lane::Library, false, move |services, _| {
        services.ai.reject_suggestion.run(&id).map_err(|error| messages::failure(error.reason).to_owned())
    })
}

/// The documents closest in meaning to `document_id`, from the Ask index; empty while it has no entry.
pub fn related(ctx: &AppContext, document_id: &DocumentId) -> Result<Vec<DocumentId>, IpcError> {
    match ctx.require_services()?.ask.related_documents.run(document_id, RELATED_LIMIT) {
        RelatedOutcome::Found(ids) => Ok(ids),
        RelatedOutcome::LibraryNotOpen | RelatedOutcome::NotFound => Ok(Vec::new()),
    }
}

/// The AI's summary of `document_id`; blocking, outside the lanes.
pub fn summarize(ctx: &AppContext, document_id: &DocumentId) -> Result<AnswerOutcome, IpcError> {
    Ok(ctx.require_services()?.ai.summarize_document.run(document_id))
}

/// The AI's answer to the last question of `conversation`; blocking, outside the lanes.
pub fn ask(ctx: &AppContext, document_id: &DocumentId, conversation: &[ChatTurn]) -> Result<AnswerOutcome, IpcError> {
    Ok(ctx.require_services()?.ai.ask_about_document.run(document_id, conversation))
}
