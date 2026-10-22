package dms.ui.review

import dms.application.assistant.model.AnswerOutcome
import dms.application.classification.model.SuggestOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.ui.session.model.UiMessageModel

/** The words for what the AI answered, including how to fix a server that does not answer. */
object AiMessages {

    fun of(outcome: SuggestOutcome, name: String): UiMessageModel {
        return when (outcome) {
            is SuggestOutcome.Suggested -> UiMessageModel(text = "The AI suggested a classification for $name")
            SuggestOutcome.LibraryNotOpen -> UiMessageModel(text = "No library is open", isError = true)
            SuggestOutcome.NotFound -> UiMessageModel(text = "$name is no longer in the library", isError = true)
            is SuggestOutcome.NotReadable -> UiMessageModel(
                text = "$name could not be read for the AI" + (outcome.detail?.let { ": $it" } ?: ""),
                isError = true,
            )

            is SuggestOutcome.Failed -> UiMessageModel(text = failure(reason = outcome.reason), isError = true)
        }
    }

    /** Why a summary or an answer is missing; `null` when there is one. */
    fun problemOf(outcome: AnswerOutcome, name: String): String? {
        return when (outcome) {
            is AnswerOutcome.Answered -> null
            AnswerOutcome.LibraryNotOpen -> "No library is open"
            AnswerOutcome.NotFound -> "$name is no longer in the library"
            is AnswerOutcome.NotReadable ->
                "$name could not be read for the AI" + (outcome.detail?.let { ": $it" } ?: "")

            is AnswerOutcome.Failed -> failure(reason = outcome.reason)
        }
    }

    fun failure(reason: ClassificationFailure): String {
        return when (reason) {
            ClassificationFailure.PROVIDER_UNAVAILABLE ->
                "The AI server does not answer. Start LM Studio's server, or check Settings › AI"

            ClassificationFailure.MODEL_CANNOT_READ_IMAGES ->
                "The model cannot read images. Load a vision model, such as Gemma 3 or Qwen2.5-VL"

            ClassificationFailure.INVALID_RESPONSE -> "The AI's answer could not be understood; try again"
            ClassificationFailure.TIMEOUT -> "The AI took too long; try again, or raise the timeout in Settings › AI"
        }
    }
}
