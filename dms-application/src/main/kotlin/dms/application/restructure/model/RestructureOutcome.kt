package dms.application.restructure.model

import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel

sealed interface RestructureOutcome {

    /**
     * [moved] documents are in their new place in the primary and the catalog; [failed] moves were not
     * possible, for example because the file is gone or the target appeared in the meantime, and
     * [skipped] documents of a reversal were changed since. [pendingFolders] wait in the journal.
     * [record] names the plan file that can reverse what was done. [emptyFolders] were left without
     * documents and may be removed. A [cancelled] run committed what it had done.
     */
    data class Applied(
        val moved: Int,
        val failed: List<PlannedMoveModel>,
        val skipped: Int,
        val pendingFolders: Set<LibraryFolderIdModel>,
        val record: String?,
        val emptyFolders: List<RelativePathModel>,
        val cancelled: Boolean,
    ) : RestructureOutcome

    data object LibraryNotOpen : RestructureOutcome

    /** The plan file to reverse does not exist or cannot be read. */
    data object RecordNotFound : RestructureOutcome
}
