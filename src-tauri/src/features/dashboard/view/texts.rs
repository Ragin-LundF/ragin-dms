//! The title and the detail line of every kind of task (Kotlin `TaskTexts`).

use dms_domain::catalog::CatalogMergeConflict;
use dms_domain::change::NewFile;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory};
use dms_domain::contract::ProposalReason;
use dms_domain::iso_date;
use dms_domain::rule::RuleProposal;
use dms_domain::task::OpenTask;

use super::actions::tag_list;
use super::context::TaskContext;
use crate::features::bills::view::{amount_text, expected_title, rhythm_words};
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
        OpenTask::BillOverdue { document_id, due } => {
            format!("Overdue since {}: {}", iso_date(*due), context.document_path(document_id))
        }
        OpenTask::ExpectedDocumentMissing(expected) => format!(
            "{} from {} expected by {}",
            expected_title(expected, context.catalog()),
            expected.correspondent,
            iso_date(expected.expected_on)
        ),
        OpenTask::ReviewContractProposal(proposal) => format!("Contract with {}?", proposal.correspondent),
        OpenTask::ContractCancelBy { contract_id, date } => {
            format!("Cancel by {}: {}", iso_date(*date), contract_name(contract_id, context))
        }
        OpenTask::FollowUpDue { case_id, date, .. } => {
            format!("Follow-up on {}: {}", iso_date(*date), case_title(case_id, context))
        }
        OpenTask::CaseWaitingTooLong { case_id, .. } => format!("No answer yet: {}", case_title(case_id, context)),
        OpenTask::DamagedCopy { document_id, folder_id, .. } => {
            format!("Damaged copy in {}: {}", context.folder_name(folder_id), context.document_path(document_id))
        }
        OpenTask::UnreadableDocument { document_id } => format!("Unreadable: {}", context.document_path(document_id)),
        OpenTask::EmergencyFolderRefresh { due } => format!("Emergency folder: refresh due on {}", iso_date(*due)),
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
        OpenTask::BillOverdue { document_id, .. } => bill_detail(document_id, context),
        OpenTask::ExpectedDocumentMissing(expected) => format!("{}; nothing new so far", rhythm_words(expected)),
        OpenTask::ReviewContractProposal(proposal) => match proposal.reason {
            ProposalReason::DeadlineField => "A document has a cancel-by or contract end date".to_owned(),
            ProposalReason::ContractCategory => {
                format!("{} documents in a contract or insurance category", proposal.document_ids.len())
            }
            ProposalReason::RecurringBills => "Sends bills regularly".to_owned(),
        },
        OpenTask::ContractCancelBy { .. } => "Cancel by then, or the contract renews".to_owned(),
        OpenTask::FollowUpDue { case_id, follow_up_id, .. } => follow_up_text(case_id, follow_up_id, context),
        OpenTask::CaseWaitingTooLong { since, .. } => format!("Waiting for an answer since {}", iso_date(*since)),
        OpenTask::DamagedCopy { source_folder_id, .. } => {
            format!("{} has an intact copy", context.folder_name(source_folder_id))
        }
        OpenTask::UnreadableDocument { .. } => "No library folder has a copy that opens".to_owned(),
        OpenTask::EmergencyFolderRefresh { .. } => "Check that its documents and contacts are current".to_owned(),
    }
}

fn contract_name(contract_id: &dms_domain::contract::ContractId, context: &TaskContext<'_>) -> String {
    context
        .catalog()
        .contracts()
        .get(contract_id)
        .map_or_else(|| contract_id.to_string(), |contract| contract.name.clone())
}

fn follow_up_text(
    case_id: &dms_domain::cases::CaseId,
    follow_up_id: &dms_domain::cases::FollowUpId,
    context: &TaskContext<'_>,
) -> String {
    let case = context.catalog().cases().get(case_id);
    let follow_up = case.and_then(|case| case.follow_ups.iter().find(|follow_up| &follow_up.id == follow_up_id));
    follow_up.map_or_else(|| "Something to do in this case".to_owned(), |follow_up| follow_up.text.clone())
}

fn case_title(case_id: &dms_domain::cases::CaseId, context: &TaskContext<'_>) -> String {
    context.catalog().cases().get(case_id).map_or_else(|| case_id.to_string(), |case| case.title.clone())
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
        CatalogMergeConflict::ContractDiffers { kept, .. } => {
            format!("Catalog copies differed for contract {}", kept.name)
        }
        CatalogMergeConflict::CaseDiffers { kept, .. } => format!("Catalog copies differed for case {}", kept.title),
        CatalogMergeConflict::TaxYearDiffers { kept, .. } => {
            format!("Catalog copies differed for tax year {}", kept.year)
        }
        CatalogMergeConflict::OwnerDiffers { .. } => "Catalog copies differed for your details".to_owned(),
        CatalogMergeConflict::EmergencyFolderDiffers { .. } => {
            "Catalog copies differed for the emergency folder".to_owned()
        }
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
        | CatalogMergeConflict::RuleDiffers { .. }
        | CatalogMergeConflict::ContractDiffers { .. }
        | CatalogMergeConflict::CaseDiffers { .. }
        | CatalogMergeConflict::TaxYearDiffers { .. }
        | CatalogMergeConflict::OwnerDiffers { .. }
        | CatalogMergeConflict::EmergencyFolderDiffers { .. } => {
            "The newer version was kept; restore an older catalog snapshot in Settings › Archive health if you need the other".to_owned()
        }
    }
}

/// "{payee} · {amount} · not marked as paid yet".
fn bill_detail(document_id: &dms_domain::document::DocumentId, context: &TaskContext<'_>) -> String {
    let document = context.catalog().documents().get(document_id);
    let payee = document.and_then(|document| document.metadata.correspondent.clone());
    let amount = document.and_then(amount_text);
    payee.into_iter().chain(amount).chain(["Not marked as paid yet".to_owned()]).collect::<Vec<_>>().join(" · ")
}
