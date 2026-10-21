package dms.application.assistant.model

import dms.application.port.ai.model.ClassificationFailure

/** The outcome of a summary or a question about one document. */
sealed interface AnswerOutcome {

    data class Answered(val text: String, val model: String) : AnswerOutcome

    data object LibraryNotOpen : AnswerOutcome

    data object NotFound : AnswerOutcome

    data class NotReadable(val detail: String?) : AnswerOutcome

    data class Failed(val reason: ClassificationFailure, val detail: String?) : AnswerOutcome
}
