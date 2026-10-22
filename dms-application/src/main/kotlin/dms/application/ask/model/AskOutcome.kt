package dms.application.ask.model

import dms.application.port.ai.model.ClassificationFailure

sealed interface AskOutcome {

    /** The answer from [model] with the sources it cites; [usedVectors] is `false` when only words were searched. */
    data class Answered(
        val text: String,
        val citations: List<CitationModel>,
        val model: String,
        val usedVectors: Boolean,
    ) : AskOutcome

    /** Nothing in the library fits the question; the AI was not asked to make something up. */
    data object NotFound : AskOutcome

    data object LibraryNotOpen : AskOutcome

    data class Failed(val reason: ClassificationFailure, val detail: String?) : AskOutcome
}
