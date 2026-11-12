//! The commands of the cases screen, the quick look's "Cases" row and the letter dialog's case options. Reads are
//! presenters; every change is one session operation in the library lane that the window waits for.
#![allow(clippy::needless_pass_by_value)] // Tauri hands command arguments over by value

use std::sync::Arc;

use chrono::{Days, NaiveDate};
use dms_application::cases::{NewCase, NewFollowUp};
use dms_application::common::OperationOutcome;
use dms_application::search::SearchOutcome;
use dms_application::services::AppServices;
use dms_domain::cases::{CaseId, EntryDirection, FollowUpId, FollowUpSource};
use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentId;
use dms_domain::parse_iso_date;

use super::dto::{
    CaseDirectionDto, CaseStatusDto, CasesFileLetterDto, CasesFilterDto, CasesFollowUpDto, CasesNewDto,
    CasesOfDocumentDto, CasesReviewDto, CasesViewDto,
};
use super::view;
use crate::commands::AppState;
use crate::context::AppContext;
use crate::dto::IpcError;
use crate::features::document::commands::wiring;
use crate::features::document::dto::LinkedDocument;
use crate::features::letters::commands::in_lane;
use crate::messages;
use crate::session::Lane;
use crate::session::reporting::blocking;

/// The follow-up a saved letter gets: look for the answer.
const LETTER_FOLLOW_UP: &str = "Check for an answer";

#[tauri::command]
#[specta::specta]
pub fn cases_view(
    ctx: AppState<'_>,
    filter: CasesFilterDto,
    selected: Option<String>,
) -> Result<CasesViewDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let selected = selected.as_deref().map(case_id).transpose()?;
    Ok(view::present_cases(&catalog, filter, selected.as_ref(), wiring::today(&ctx)))
}

#[tauri::command]
#[specta::specta]
pub fn cases_of_document(ctx: AppState<'_>, document_id: String) -> Result<CasesOfDocumentDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    Ok(view::cases_of_document(&catalog, wiring::find_document(&catalog, &document_id)?))
}

/// The documents "Add document" offers for `case_id`, matching `query`.
#[tauri::command]
#[specta::specta]
pub fn cases_document_candidates(
    ctx: AppState<'_>,
    case_id: String,
    query: String,
) -> Result<Vec<LinkedDocument>, IpcError> {
    let catalog = ctx.require_catalog()?;
    let case = catalog.cases().get(&self::case_id(&case_id)?).ok_or_else(case_gone)?;
    Ok(view::document_candidates(&catalog, case, &query))
}

/// The case part of the review of `document_id`. The documents matching what the letter requests are found by the
/// text search, which reads files the first time; it only reads, so it waits for no lane.
#[tauri::command]
#[specta::specta]
pub async fn cases_review(ctx: AppState<'_>, document_id: String) -> Result<CasesReviewDto, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?.clone();
    let suggestion = wiring::pending_suggestion(&ctx, &document.id);
    let references: Vec<String> = suggestion
        .iter()
        .flat_map(|suggestion| &suggestion.requested_actions)
        .flat_map(|action| action.references.iter().cloned())
        .collect();
    let services = Arc::clone(ctx.require_services()?);
    let matches = blocking(move || Ok(matching_documents(&services, &references))).await?;
    Ok(view::review_cases(&catalog, &document, suggestion.as_ref(), &matches, wiring::today(&ctx)))
}

#[tauri::command]
#[specta::specta]
pub async fn cases_create(ctx: AppState<'_>, input: CasesNewDto) -> Result<String, IpcError> {
    let new = NewCase {
        title: input.title,
        correspondent: input.correspondent,
        documents: document_ids(&input.document_ids)?.into_iter().map(|id| (id, EntryDirection::Received)).collect(),
        follow_ups: Vec::new(),
    };
    create(&ctx, new).await
}

/// "Create case from linked documents": the document and every document linked to it, as received.
#[tauri::command]
#[specta::specta]
pub async fn cases_create_from_links(ctx: AppState<'_>, document_id: String) -> Result<String, IpcError> {
    let catalog = ctx.require_catalog()?;
    let document = wiring::find_document(&catalog, &document_id)?;
    // shortcut: links carry no direction, so every document starts as received; a direction switch would fix it.
    let linked = catalog.links_of(&document.id).into_iter().map(|link| link.other_than(&document.id).clone());
    let new = NewCase {
        title: view_title(document),
        correspondent: document.metadata.correspondent.clone(),
        documents: std::iter::once(document.id.clone())
            .chain(linked)
            .map(|id| (id, EntryDirection::Received))
            .collect(),
        follow_ups: Vec::new(),
    };
    create(&ctx, new).await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_add_document(
    ctx: AppState<'_>,
    case_id: String,
    document_id: String,
    direction: CaseDirectionDto,
) -> Result<(), IpcError> {
    let (case, document) = (self::case_id(&case_id)?, wiring::document_id(&document_id)?);
    let direction = direction_of(direction);
    change(&ctx, "Adding to the case", move |services| services.cases.add_to_case.run(&case, &document, direction))
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_remove_document(ctx: AppState<'_>, case_id: String, document_id: String) -> Result<(), IpcError> {
    let (case, document) = (self::case_id(&case_id)?, wiring::document_id(&document_id)?);
    change(&ctx, "Removing from the case", move |services| services.cases.remove_from_case.run(&case, &document)).await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_set_status(ctx: AppState<'_>, case_id: String, status: CaseStatusDto) -> Result<(), IpcError> {
    let case = self::case_id(&case_id)?;
    let status = view::status_of(status);
    change(&ctx, "Changing the case", move |services| services.cases.set_case_status.run(&case, status)).await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_add_note(ctx: AppState<'_>, case_id: String, text: String) -> Result<(), IpcError> {
    let case = self::case_id(&case_id)?;
    change(&ctx, "Adding a note", move |services| services.cases.add_case_note.run(&case, &text)).await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_add_follow_up(
    ctx: AppState<'_>,
    case_id: String,
    follow_up: CasesFollowUpDto,
) -> Result<(), IpcError> {
    let case = self::case_id(&case_id)?;
    let new = new_follow_up(&follow_up, FollowUpSource::Manual)?;
    change(&ctx, "Adding a follow-up", move |services| services.cases.add_follow_up.run(&case, &new)).await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_complete_follow_up(
    ctx: AppState<'_>,
    case_id: String,
    follow_up_id: String,
) -> Result<(), IpcError> {
    let (case, follow_up) = (self::case_id(&case_id)?, self::follow_up_id(&follow_up_id)?);
    change(&ctx, "Completing the follow-up", move |services| services.cases.complete_follow_up.run(&case, &follow_up))
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn cases_snooze_follow_up(
    ctx: AppState<'_>,
    case_id: String,
    follow_up_id: String,
    days: u16,
) -> Result<(), IpcError> {
    let (case, follow_up) = (self::case_id(&case_id)?, self::follow_up_id(&follow_up_id)?);
    change(&ctx, "Snoozing the follow-up", move |services| services.cases.snooze_follow_up.run(&case, &follow_up, days))
        .await
}

/// Deletes the case once the window asked; its documents stay.
#[tauri::command]
#[specta::specta]
pub async fn cases_delete(ctx: AppState<'_>, case_id: String, confirmed: bool) -> Result<(), IpcError> {
    let case = self::case_id(&case_id)?;
    change(&ctx, "Deleting the case", move |services| services.cases.delete_case.run(&case, confirmed)).await
}

/// After a letter was saved: into its case as sent, or into a new case with its sources, and a follow-up.
#[tauri::command]
#[specta::specta]
pub async fn cases_file_letter(ctx: AppState<'_>, input: CasesFileLetterDto) -> Result<(), IpcError> {
    let catalog = ctx.require_catalog()?;
    let letter = wiring::find_document(&catalog, &input.letter_id)?.clone();
    let sources = document_ids(&input.source_ids)?;
    let existing = input.case_id.as_deref().map(case_id).transpose()?;
    let follow_up = input.follow_up_days.map(|days| NewFollowUp {
        text: LETTER_FOLLOW_UP.to_owned(),
        date: days_after(wiring::today(&ctx), days),
        source: FollowUpSource::Manual,
    });
    if existing.is_none() && !input.start_case {
        return Ok(());
    }
    in_lane(&ctx, "Filing the letter in a case", Lane::Library, move |services| {
        let (case, outcome) = match existing {
            Some(case) => {
                let outcome = services.cases.add_to_case.run(&case, &letter.id, EntryDirection::Sent);
                (case, outcome)
            }
            None => start_case_with(services, &letter, sources),
        };
        match (outcome, follow_up) {
            (OperationOutcome::Done { .. }, Some(follow_up)) => services.cases.add_follow_up.run(&case, &follow_up),
            (outcome, _) => outcome,
        }
    })
    .await
    .and_then(|outcome| refused(&outcome))
}

/// A new case for a saved letter: its sources as received, the letter as sent.
fn start_case_with(
    services: &AppServices,
    letter: &dms_domain::document::Document,
    sources: Vec<DocumentId>,
) -> (CaseId, OperationOutcome) {
    let documents = sources
        .into_iter()
        .map(|id| (id, EntryDirection::Received))
        .chain([(letter.id.clone(), EntryDirection::Sent)])
        .collect();
    let new = NewCase {
        title: view_title(letter),
        correspondent: letter.metadata.correspondent.clone(),
        documents,
        follow_ups: Vec::new(),
    };
    let created = services.cases.create_case.run(&new);
    (created.case_id, created.outcome)
}

/// What the review's approval also does with cases, run in its operation after the approval succeeded.
pub fn apply_review_choice(
    services: &AppServices,
    catalog: &Catalog,
    document: &DocumentId,
    choice: &super::dto::CasesReviewChoiceDto,
) -> OperationOutcome {
    use super::dto::CasesReviewChoiceDto;
    let source = FollowUpSource::Ai { document_id: document.clone() };
    let follow_ups = |follow_ups: &[CasesFollowUpDto]| -> Vec<NewFollowUp> {
        follow_ups.iter().filter_map(|follow_up| new_follow_up(follow_up, source.clone()).ok()).collect()
    };
    match choice {
        CasesReviewChoiceDto::NewCase { title, follow_ups: checked } => {
            let correspondent =
                catalog.documents().get(document).and_then(|document| document.metadata.correspondent.clone());
            let new = NewCase {
                title: title.clone(),
                correspondent,
                documents: vec![(document.clone(), EntryDirection::Received)],
                follow_ups: follow_ups(checked),
            };
            services.cases.create_case.run(&new).outcome
        }
        CasesReviewChoiceDto::AddToCase { case_id: raw, follow_ups: checked } => {
            let Ok(case) = case_id(raw) else {
                return OperationOutcome::Rejected(dms_domain::catalog::CatalogRejection::CaseNotFound);
            };
            let added = services.cases.add_to_case.run(&case, document, EntryDirection::Received);
            follow_ups(checked).iter().fold(added, |outcome, follow_up| match outcome {
                OperationOutcome::Done { .. } => services.cases.add_follow_up.run(&case, follow_up),
                refused => refused,
            })
        }
    }
}

async fn create(ctx: &Arc<AppContext>, new: NewCase) -> Result<String, IpcError> {
    let created =
        in_lane(ctx, "Starting a case", Lane::Library, move |services| services.cases.create_case.run(&new)).await?;
    refused(&created.outcome)?;
    Ok(created.case_id.to_string())
}

async fn change(
    ctx: &Arc<AppContext>,
    label: &str,
    job: impl FnOnce(&AppServices) -> OperationOutcome + Send + 'static,
) -> Result<(), IpcError> {
    let outcome = in_lane(ctx, label, Lane::Library, job).await?;
    refused(&outcome)
}

/// A refused change as the error the window reports.
fn refused(outcome: &OperationOutcome) -> Result<(), IpcError> {
    match outcome {
        OperationOutcome::Done { .. } => Ok(()),
        refused => Err(IpcError::invalid(messages::of(refused, "").text)),
    }
}

/// The documents whose text contains one of `references`, in the order found.
fn matching_documents(services: &AppServices, references: &[String]) -> Vec<DocumentId> {
    let mut found: Vec<DocumentId> = Vec::new();
    for reference in references {
        if let SearchOutcome::Found(ids) = services.library.search_contents.run(reference) {
            found.extend(ids.into_iter().filter(|id| !found.contains(id)).collect::<Vec<_>>());
        }
    }
    found
}

fn new_follow_up(follow_up: &CasesFollowUpDto, source: FollowUpSource) -> Result<NewFollowUp, IpcError> {
    let date = parse_iso_date(&follow_up.date).ok_or_else(|| IpcError::invalid("The date must be YYYY-MM-DD"))?;
    Ok(NewFollowUp { text: follow_up.text.clone(), date, source })
}

fn days_after(today: NaiveDate, days: u16) -> NaiveDate {
    today.checked_add_days(Days::new(u64::from(days))).unwrap_or(today)
}

fn view_title(document: &dms_domain::document::Document) -> String {
    crate::features::document::view::details::display_name(document).to_owned()
}

fn direction_of(direction: CaseDirectionDto) -> EntryDirection {
    match direction {
        CaseDirectionDto::Received => EntryDirection::Received,
        CaseDirectionDto::Sent => EntryDirection::Sent,
    }
}

fn case_id(raw: &str) -> Result<CaseId, IpcError> {
    CaseId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))
}

fn follow_up_id(raw: &str) -> Result<FollowUpId, IpcError> {
    FollowUpId::new(raw).map_err(|error| IpcError::invalid(error.to_string()))
}

fn document_ids(raw: &[String]) -> Result<Vec<DocumentId>, IpcError> {
    raw.iter().map(|id| wiring::document_id(id)).collect()
}

fn case_gone() -> IpcError {
    IpcError::invalid("The case no longer exists")
}
