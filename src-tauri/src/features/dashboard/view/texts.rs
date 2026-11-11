//! The title and the detail line of every kind of task (Kotlin `TaskTexts`).

use dms_domain::catalog::CatalogMergeConflict;
use dms_domain::change::NewFile;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory};
use dms_domain::iso_date;
use dms_domain::rule::RuleProposal;
use dms_domain::task::OpenTask;

use super::actions::tag_list;
use super::context::TaskContext;
use crate::features::dashboard::format::file_size;

pub fn title(task: &OpenTask, context: &TaskContext<'_>) -> String {
    match task {
        OpenTask::ReviewSuggestion(suggestion) => {
            format!("Suggestion for {}", context.document_path(&suggestion.document_id))
        }
        OpenTask::AdoptNewFile(change) => {
            if context.original_of(change).is_none() {
                format!("New file {}", change.path)
            } else {
                format!("Copy of a document: {}", change.path)
            }
        }
        OpenTask::RestoreOrDelete(change) => format!("Missing {}", change.path),
        OpenTask::ReviewModification(change) => format!("Changed outside the tool: {}", change.path),
        OpenTask::ConfirmMove(change) => {
            format!("Other name in {}: {}", context.folder_name(&change.folder_id), change.path)
        }
        OpenTask::ResolveDuplicates { document_ids, .. } => format!("{} copies of the same file", document_ids.len()),
        OpenTask::ResolveContentConflict(change) => format!("Different content: {}", change.path),
        OpenTask::ReviewCatalogMerge(conflict) => merge_title(conflict),
        OpenTask::RetryFailedOperations { folder_id, count } => {
            format!("{count} operation(s) waiting for {}", context.folder_name(folder_id))
        }
        OpenTask::DeadlineDue { document_id, field_key, date } => {
            format!("{} on {}: {}", context.field_label(field_key), iso_date(*date), context.document_path(document_id))
        }
        OpenTask::RetentionDue { document_id, .. } => {
            format!("Retention ended: {}", context.document_path(document_id))
        }
        OpenTask::RestructureSuggested { folder, file_count } => {
            format!("{file_count} files without structure in {folder}")
        }
        OpenTask::ReviewLearnedRule(proposal) => format!("Rule for {}", proposal.correspondent),
    }
}

pub fn detail(task: &OpenTask, context: &TaskContext<'_>) -> String {
    match task {
        OpenTask::ReviewSuggestion(suggestion) => suggestion_detail(suggestion, context),
        OpenTask::AdoptNewFile(change) => new_file_detail(change, context),
        OpenTask::RestoreOrDelete(change) => format!(
            "Missing in {}; still in {}",
            context.folder_names(&change.missing_in),
            context.folder_names(&change.present_in)
        ),
        OpenTask::ReviewModification(change) => {
            format!("The content in {} differs from the catalog", context.folder_name(&change.folder_id))
        }
        OpenTask::ConfirmMove(change) => format!("The catalog and the other folders call it {}", change.previous_path),
        OpenTask::ResolveDuplicates { .. } => {
            "Same size and SHA-256 checksum. Keep one; the others go to the trash in every folder.".to_owned()
        }
        OpenTask::ResolveContentConflict(change) => format!(
            "{} versions in {}",
            change.content_hashes.len(),
            context.folder_names(change.content_hashes.keys())
        ),
        OpenTask::ReviewCatalogMerge(conflict) => merge_detail(conflict),
        OpenTask::RetryFailedOperations { .. } => "They are applied in order once the folder is reachable".to_owned(),
        OpenTask::DeadlineDue { .. } => "Dismiss once it is handled; a new date brings the task back".to_owned(),
        OpenTask::RetentionDue { date, .. } => format!(
            "You chose to keep it until {}. Move it to the trash, or keep it and mark this as done.",
            iso_date(*date)
        ),
        OpenTask::RestructureSuggested { .. } => "The restructure assistant can sort them into subfolders".to_owned(),
        OpenTask::ReviewLearnedRule(proposal) => learned_rule_detail(proposal, context),
    }
}

fn new_file_detail(change: &NewFile, context: &TaskContext<'_>) -> String {
    let place = format!("In {}, {}", context.folder_names(&change.present_in), file_size(change.size));
    match context.original_of(change) {
        Some(original) => format!("{place}; same size and checksum as {}", original.path),
        None => place,
    }
}

fn suggestion_detail(suggestion: &ClassificationSuggestion, context: &TaskContext<'_>) -> String {
    let category = suggestion.category.as_ref().map(|category| match category {
        SuggestedCategory::Existing(id) => context.category_name(id),
        SuggestedCategory::New { name, .. } => format!("{name} (new)"),
    });
    let mut parts = Vec::new();
    if let Some(title) = &suggestion.title {
        parts.push(format!("Title: {title}"));
    }
    if let Some(category) = category {
        parts.push(format!("Category: {category}"));
    }
    if !suggestion.tags.is_empty() {
        parts.push(format!("Tags: {}", tag_list(suggestion.tags.iter().map(|tag| tag.name.value()))));
    }
    if let Some(file_name) = &suggestion.file_name {
        parts.push(format!("Rename → {file_name}"));
    }
    parts.join(" · ")
}

fn learned_rule_detail(proposal: &RuleProposal, context: &TaskContext<'_>) -> String {
    let mut parts = Vec::new();
    if let Some(category) = &proposal.category_id {
        parts.push(format!("Category: {}", context.category_name(category)));
    }
    if !proposal.tags.is_empty() {
        parts.push(format!("Tags: {}", tag_list(proposal.tags.iter().map(dms_domain::taxonomy::TagName::value))));
    }
    format!("From {} approved documents · {}", proposal.document_count, parts.join(" · "))
}

fn merge_title(conflict: &CatalogMergeConflict) -> String {
    match conflict {
        CatalogMergeConflict::DocumentDiffers { kept, .. } => format!("Catalog copies differed for {}", kept.path),
        CatalogMergeConflict::PathClash { kept, .. } => format!("Two documents claimed {}", kept.path),
        CatalogMergeConflict::CategoryDiffers { kept, .. } => {
            format!("Catalog copies differed for category {}", kept.name)
        }
        CatalogMergeConflict::FieldDefinitionDiffers { kept, .. } => {
            format!("Catalog copies differed for field {}", kept.label)
        }
        CatalogMergeConflict::SavedSearchDiffers { kept, .. } => {
            format!("Catalog copies differed for smart folder {}", kept.name)
        }
        CatalogMergeConflict::RuleDiffers { kept, .. } => format!("Catalog copies differed for rule {}", kept.name),
    }
}

fn merge_detail(conflict: &CatalogMergeConflict) -> String {
    match conflict {
        CatalogMergeConflict::DocumentDiffers { .. } => "The newer entry was kept".to_owned(),
        CatalogMergeConflict::PathClash { .. } => {
            "The newer entry was kept; the other one was dropped from the catalog".to_owned()
        }
        CatalogMergeConflict::CategoryDiffers { kept, discarded } => {
            format!("Kept \"{}\", dropped \"{}\"", kept.name, discarded.name)
        }
        CatalogMergeConflict::FieldDefinitionDiffers { .. }
        | CatalogMergeConflict::SavedSearchDiffers { .. }
        | CatalogMergeConflict::RuleDiffers { .. } => "The newer version was kept".to_owned(),
    }
}
