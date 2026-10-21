package dms.ui.review

import dms.application.classification.model.SuggestOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.ui.session.model.UiMessage

/** The words for what the AI answered, including how to fix a server that does not answer. */
object AiMessages {

    fun of(outcome: SuggestOutcome, name: String): UiMessage {
        return when (outcome) {
            is SuggestOutcome.Suggested -> UiMessage(text = "The AI suggested a classification for $name")
            SuggestOutcome.LibraryNotOpen -> UiMessage(text = "No library is open", isError = true)
            SuggestOutcome.NotFound -> UiMessage(text = "$name is no longer in the library", isError = true)
            is SuggestOutcome.NotReadable -> UiMessage(
                text = "$name could not be read for the AI" + (outcome.detail?.let { ": $it" } ?: ""),
                isError = true,
            )

            is SuggestOutcome.Failed -> UiMessage(text = failure(reason = outcome.reason), isError = true)
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
