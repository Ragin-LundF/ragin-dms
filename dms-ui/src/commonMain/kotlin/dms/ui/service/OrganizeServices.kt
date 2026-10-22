package dms.ui.service

import dms.application.export.ExportDocuments
import dms.application.inbox.CollectInbox
import dms.application.inbox.ConfigureInbox
import dms.application.inbox.FileInboxDocument
import dms.application.port.export.ExportWriter
import dms.application.port.storage.FolderWatcher
import dms.application.restructure.ApplyRestructurePlan
import dms.application.restructure.PlanRestructure
import dms.application.restructure.RemoveEmptyFolders
import dms.application.restructure.ReverseRestructure
import dms.application.rule.AcceptRuleProposal

/** Getting documents in, filed and out: the inbox, the restructure assistant, learned rules and exports. */
class OrganizeServices(
    val configureInbox: ConfigureInbox,
    val collectInbox: CollectInbox,
    val fileInboxDocument: FileInboxDocument,
    val folderWatcher: FolderWatcher,
    val planRestructure: PlanRestructure,
    val applyRestructurePlan: ApplyRestructurePlan,
    val reverseRestructure: ReverseRestructure,
    val removeEmptyFolders: RemoveEmptyFolders,
    val acceptRuleProposal: AcceptRuleProposal,
    val exportDocuments: ExportDocuments,
    val exportWriter: ExportWriter,
)
