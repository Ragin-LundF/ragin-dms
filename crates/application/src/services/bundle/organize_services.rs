use std::sync::Arc;

use crate::bill::{MarkBillPaid, MarkBillUnpaid, SettleOlderBills, SuggestFromEInvoice};
use crate::contract::{
    AttachDocumentsToContract, DeleteContract, DismissContractProposal, EndContract, ExtractContractTerms, SaveContract,
};
use crate::emergency::{
    CreateEmergencyOverview, EditEmergencyFolder, ExportEmergencyFolder, MarkEmergencyFolderRefreshed,
};
use crate::expectation::SnoozeExpectedDocument;
use crate::export::ExportDocuments;
use crate::inbox::{CollectInbox, ConfigureInbox, FileInboxDocument};
use crate::mail::ImportEmails;
use crate::port::export::ExportWriter;
use crate::port::storage::FolderWatcher;
use crate::restructure::{
    ApplyRestructurePlan, ListRestructureHistory, PlanRestructure, RemoveEmptyFolders, ReverseRestructure,
};
use crate::rule::AcceptRuleProposal;
use crate::tax::{AssignTax, OpenTaxYear, UpdateTaxItem};

/// Getting documents in, filed and out: the inbox, email import, the restructure assistant, learned rules, exports
/// and the emergency folder; bills to pay, expected documents and contracts.
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
    pub open_tax_year: Arc<OpenTaxYear>,
    pub update_tax_item: Arc<UpdateTaxItem>,
    pub assign_tax: Arc<AssignTax>,
    pub mark_bill_paid: Arc<MarkBillPaid>,
    pub mark_bill_unpaid: Arc<MarkBillUnpaid>,
    pub settle_older_bills: Arc<SettleOlderBills>,
    pub suggest_from_e_invoice: Arc<SuggestFromEInvoice>,
    pub snooze_expected_document: Arc<SnoozeExpectedDocument>,
    pub save_contract: Arc<SaveContract>,
    pub end_contract: Arc<EndContract>,
    pub delete_contract: Arc<DeleteContract>,
    pub attach_documents_to_contract: Arc<AttachDocumentsToContract>,
    pub dismiss_contract_proposal: Arc<DismissContractProposal>,
    pub extract_contract_terms: Arc<ExtractContractTerms>,
    pub import_emails: Arc<ImportEmails>,
    pub edit_emergency_folder: Arc<EditEmergencyFolder>,
    pub mark_emergency_folder_refreshed: Arc<MarkEmergencyFolderRefreshed>,
    pub create_emergency_overview: Arc<CreateEmergencyOverview>,
    pub export_emergency_folder: Arc<ExportEmergencyFolder>,
}
