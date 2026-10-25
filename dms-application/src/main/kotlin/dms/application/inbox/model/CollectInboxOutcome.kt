package dms.application.inbox.model

import dms.domain.change.DetectedChange

sealed interface CollectInboxOutcome {

    /**
     * The changes that became inbox documents, so the caller can drop them from the scan; none while the
     * inbox is off.
     */
    data class Collected(val changes: List<DetectedChange.NewFile>) : CollectInboxOutcome

    data object LibraryNotOpen : CollectInboxOutcome
}
