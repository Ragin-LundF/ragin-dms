package dms.application.classification.model

import dms.application.port.ai.model.ClassificationFailure
import dms.domain.classification.model.ClassificationSuggestionModel

sealed interface SuggestOutcome {

    /** The suggestion waits on the dashboard for the user's review. */
    data class Suggested(val suggestion: ClassificationSuggestionModel) : SuggestOutcome

    data object LibraryNotOpen : SuggestOutcome

    data object NotFound : SuggestOutcome

    /** The file type is never sent to the AI, or the file could not be read. [detail] is technical. */
    data class NotReadable(val detail: String?) : SuggestOutcome

    data class Failed(val reason: ClassificationFailure, val detail: String?) : SuggestOutcome
}
