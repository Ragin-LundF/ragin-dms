use std::sync::Arc;

use crate::export::ExportDocuments;
use crate::inbox::{CollectInbox, ConfigureInbox, FileInboxDocument};
use crate::port::export::ExportWriter;
use crate::port::storage::FolderWatcher;
use crate::restructure::{
    ApplyRestructurePlan, ListRestructureHistory, PlanRestructure, RemoveEmptyFolders, ReverseRestructure,
};
use crate::rule::AcceptRuleProposal;

/// Getting documents in, filed and out: the inbox, the restructure assistant, learned rules and exports.
pub struct OrganizeServices {
    pub configure_inbox: Arc<ConfigureInbox>,
    pub collect_inbox: Arc<CollectInbox>,
    pub file_inbox_document: Arc<FileInboxDocument>,
    pub folder_watcher: Arc<dyn FolderWatcher>,
    pub plan_restructure: Arc<PlanRestructure>,
    pub apply_restructure_plan: Arc<ApplyRestructurePlan>,
    pub reverse_restructure: Arc<ReverseRestructure>,
    pub list_restructure_history: Arc<ListRestructureHistory>,
    pub remove_empty_folders: Arc<RemoveEmptyFolders>,
    pub accept_rule_proposal: Arc<AcceptRuleProposal>,
    pub export_documents: Arc<ExportDocuments>,
    pub export_writer: Arc<dyn ExportWriter>,
}
