//! Presenters of the restructure screen (the deriving part of Kotlin's `RestructureViewModel`).

use std::cmp::Ordering;
use std::collections::HashSet;

use dms_application::restructure::{RestructureOutcome, RestructureScope};
use dms_domain::catalog::Catalog;
use dms_domain::library::RelativePath;
use dms_domain::restructure::{PlaceholderName, PlannedMove, RestructurePlan, SkipReason, TemplateProblem};
use dms_domain::text;

use super::dto::{ApplyPreviewDto, MoveGroupDto, PlanDto, RestructureScopeDto, ScopeOptionDto};
use crate::dto::IpcError;
use crate::session::UiMessage;

pub const DEFAULT_TEMPLATE: &str = "{category}/{year|Undated}";
const SAMPLES: usize = 3;
const AFFECTED_SHOWN: usize = 12;

/// "Whole library", then one entry per smart folder, then every folder that holds documents (sorted ignoring case).
pub fn scope_options(catalog: &Catalog) -> Vec<ScopeOptionDto> {
    let whole = ScopeOptionDto {
        label: "Whole library".to_owned(),
        scope: RestructureScopeDto::Folder { path: String::new() },
    };
    let searches = catalog.settings().saved_searches.iter().map(|search| ScopeOptionDto {
        label: format!("Smart folder: {}", search.name),
        scope: RestructureScopeDto::SmartFolder { id: search.id.value().to_owned() },
    });
    let folders = folder_paths(catalog)
        .into_iter()
        .map(|folder| ScopeOptionDto { label: folder.clone(), scope: RestructureScopeDto::Folder { path: folder } });
    std::iter::once(whole).chain(searches).chain(folders).collect()
}

/// The scope a picker entry stands for.
pub fn resolve_scope(catalog: &Catalog, scope: &RestructureScopeDto) -> Result<RestructureScope, IpcError> {
    match scope {
        RestructureScopeDto::Folder { path } if path.is_empty() => Ok(RestructureScope::Folder(RelativePath::root())),
        RestructureScopeDto::Folder { path } => {
            RelativePath::of(path).map(RestructureScope::Folder).map_err(|error| IpcError::invalid(error.to_string()))
        }
        RestructureScopeDto::SmartFolder { id } => catalog
            .settings()
            .saved_searches
            .iter()
            .find(|search| search.id.value() == id)
            .map(|search| RestructureScope::Query(search.query.clone()))
            .ok_or_else(|| IpcError::invalid("The smart folder no longer exists")),
    }
}

/// The chips below the template field: every placeholder, the field one with an example key.
pub fn placeholders() -> Vec<String> {
    [
        PlaceholderName::Category,
        PlaceholderName::CategoryTop,
        PlaceholderName::Year,
        PlaceholderName::Month,
        PlaceholderName::Correspondent,
        PlaceholderName::Kind,
        PlaceholderName::Field,
    ]
    .into_iter()
    .map(|placeholder| match placeholder {
        PlaceholderName::Field => "field:due".to_owned(),
        other => other.token().to_owned(),
    })
    .collect()
}

pub fn problem_text(problem: TemplateProblem) -> &'static str {
    match problem {
        TemplateProblem::Empty => "Write a template, such as {category}/{year}",
        TemplateProblem::UnclosedBrace => "A { has no closing }",
        TemplateProblem::UnknownPlaceholder => "Unknown placeholder; use the chips below",
        TemplateProblem::InvalidFieldKey => "Write fields as {field:key}, such as {field:due}",
        TemplateProblem::InvalidFolderName => "A folder name is not allowed or contains one of < > : \" \\ | ? *",
    }
}

pub fn present_plan(plan: &RestructurePlan) -> PlanDto {
    let mut folders: Vec<(String, Vec<&PlannedMove>)> = Vec::new();
    for planned in &plan.moves {
        let folder = target_folder(planned);
        match folders.iter_mut().find(|(known, _)| *known == folder) {
            Some((_, moves)) => moves.push(planned),
            None => folders.push((folder, vec![planned])),
        }
    }
    folders.sort_by(|(first, _), (second, _)| text::compare(first, second));
    let groups = folders
        .into_iter()
        .map(|(folder, moves)| MoveGroupDto {
            folder,
            count: count(moves.len()),
            samples: moves
                .iter()
                .take(SAMPLES)
                .map(|it| format!("{} → {}", it.from.value(), it.to.file_name()))
                .collect(),
        })
        .collect();
    PlanDto {
        groups,
        unchanged: count(plan.unchanged.len()),
        skipped: skipped_counts(plan).into_iter().map(|(reason, n)| format!("{n} {}", skip_text(reason))).collect(),
        renamed: plan
            .moves
            .iter()
            .filter(|planned| plan.renamed_for_clash.contains(&planned.document_id))
            .map(|planned| format!("{} → {}", planned.from.file_name(), planned.to.value()))
            .collect(),
    }
}

/// The plan with only the moves into the ticked target folders.
pub fn chosen_plan(plan: &RestructurePlan, included_folders: &[String]) -> RestructurePlan {
    let included: HashSet<&str> = included_folders.iter().map(String::as_str).collect();
    plan.only(|planned| included.contains(target_folder(planned).as_str()))
}

/// The confirmation's numbers: the first twelve moves as `from → to`, then "… and n more".
pub fn apply_preview(chosen: &RestructurePlan) -> ApplyPreviewDto {
    let mut affected: Vec<String> = chosen
        .moves
        .iter()
        .take(AFFECTED_SHOWN)
        .map(|planned| format!("{} → {}", planned.from.value(), planned.to.value()))
        .collect();
    if chosen.moves.len() > AFFECTED_SHOWN {
        affected.push(format!("… and {} more", chosen.moves.len() - AFFECTED_SHOWN));
    }
    ApplyPreviewDto { moves: count(chosen.moves.len()), affected }
}

/// What an applied or reversed restructure tells the user: the message, error lines for the log, and the folders
/// it left empty.
#[derive(Debug, PartialEq, Eq)]
pub struct RestructureReport {
    pub message: UiMessage,
    pub log_errors: Vec<String>,
    pub empty_folders: Vec<String>,
}

pub fn report_of(outcome: &RestructureOutcome) -> RestructureReport {
    let only = |text: &str| RestructureReport {
        message: UiMessage::error(text),
        log_errors: Vec::new(),
        empty_folders: Vec::new(),
    };
    match outcome {
        RestructureOutcome::LibraryNotOpen => only("No library is open"),
        RestructureOutcome::RecordNotFound => only("The record of this restructure cannot be read"),
        RestructureOutcome::Applied {
            moved, failed, not_catalogued, skipped, pending_folders, empty_folders, ..
        } => {
            let mut parts = vec![format!("Moved {moved} documents")];
            if !failed.is_empty() {
                parts.push(format!("{} could not be moved", failed.len()));
            }
            if !not_catalogued.is_empty() {
                parts.push(format!("{} moved, but the catalog could not be saved; run a scan", not_catalogued.len()));
            }
            if *skipped > 0 {
                parts.push(format!("{skipped} changed since and stayed"));
            }
            if !pending_folders.is_empty() {
                parts.push(format!("{} folders wait in the journal", pending_folders.len()));
            }
            let text = parts.join("; ");
            let is_error = !failed.is_empty() || !not_catalogued.is_empty();
            let log_errors = failed
                .iter()
                .map(|planned| format!("Not moved: {}", planned.from.value()))
                .chain(not_catalogued.iter().map(|planned| format!("Moved, not catalogued: {}", planned.to.value())))
                .collect();
            RestructureReport {
                message: if is_error { UiMessage::error(text) } else { UiMessage::info(text) },
                log_errors,
                empty_folders: empty_folders.iter().map(|folder| folder.value().to_owned()).collect(),
            }
        }
    }
}

fn target_folder(planned: &PlannedMove) -> String {
    planned.to.parent().map(|parent| parent.value().to_owned()).unwrap_or_default()
}

/// Kotlin's `groupingBy { reason }.eachCount()`: counts in the order each reason first appears.
fn skipped_counts(plan: &RestructurePlan) -> Vec<(SkipReason, usize)> {
    let mut counts: Vec<(SkipReason, usize)> = Vec::new();
    for skipped in &plan.skipped {
        match counts.iter_mut().find(|(reason, _)| *reason == skipped.reason) {
            Some((_, n)) => *n += 1,
            None => counts.push((skipped.reason, 1)),
        }
    }
    counts
}

fn skip_text(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::MissingCategory => "without category",
        SkipReason::MissingDate => "without date",
        SkipReason::MissingCorrespondent => "without correspondent",
        SkipReason::MissingField => "without the field value",
        SkipReason::InInbox => "in the inbox",
    }
}

/// Every folder above a document (not the root), distinct, in Java's `CASE_INSENSITIVE_ORDER`.
fn folder_paths(catalog: &Catalog) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut folders = Vec::new();
    for document in catalog.documents().values() {
        let mut parent = document.path.parent();
        while let Some(folder) = parent.filter(|folder| !folder.is_root()) {
            if seen.insert(folder.value().to_owned()) {
                folders.push(folder.value().to_owned());
            }
            parent = folder.parent();
        }
    }
    folders.sort_by(|first, second| case_insensitive(first, second));
    folders
}

/// Java's `String.CASE_INSENSITIVE_ORDER`: per UTF-16 unit, upper case first, then lower case.
fn case_insensitive(first: &str, second: &str) -> Ordering {
    let fold = |unit: u16| {
        // shortcut: case mapping of BMP characters only, which is what Java's per-char mapping covers too.
        let Some(character) = char::from_u32(u32::from(unit)) else { return unit };
        let upper = single(character.to_uppercase()).unwrap_or(character);
        let lower = single(upper.to_lowercase()).unwrap_or(upper);
        u16::try_from(u32::from(lower)).unwrap_or(unit)
    };
    first.encode_utf16().map(fold).cmp(second.encode_utf16().map(fold))
}

fn single(mut mapped: impl Iterator<Item = char>) -> Option<char> {
    let first = mapped.next()?;
    mapped.next().is_none().then_some(first)
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
