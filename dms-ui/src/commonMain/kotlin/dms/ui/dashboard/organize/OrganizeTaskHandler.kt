package dms.ui.dashboard.organize

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.learn.model.RuleProposalModel

/** The answers to the tasks of 1.1's organizing features: restructure, retention and learned rules. */
interface OrganizeTaskHandler {

    /** Opens the restructure assistant on [folder]. */
    fun restructure(folder: RelativePathModel)

    /** Asks first, then moves documents whose retention ended to the trash in every folder. */
    fun trashRetained(documentIds: List<DocumentIdModel>)

    /** Marks the ended retention of documents as done; they stay ordinary documents. */
    fun settleRetention(documentIds: List<DocumentIdModel>)

    /** Adds the proposed rule; rejecting it is dismissing its task. */
    fun acceptRule(proposal: RuleProposalModel)
}
