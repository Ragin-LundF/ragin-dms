package dms.application.restructure.model

sealed interface RestructureHistoryOutcome {

    /** The names of the restructure records, newest first; each can be reversed. */
    data class Listed(val names: List<String>) : RestructureHistoryOutcome

    data object LibraryNotOpen : RestructureHistoryOutcome
}
