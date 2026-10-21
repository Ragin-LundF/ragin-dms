package dms.application.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.application.assistant.model.ReadingResult
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.assistant.model.AssistantAnswer

/** Maps reading problems and AI answers to the outcomes the UI shows. */
object AssistantOutcomes {

    fun of(problem: ReadingResult): AnswerOutcome {
        return when (problem) {
            is ReadingResult.Read -> error("A document that was read is no problem")
            ReadingResult.LibraryNotOpen -> AnswerOutcome.LibraryNotOpen
            ReadingResult.NotFound -> AnswerOutcome.NotFound
            is ReadingResult.NotReadable -> AnswerOutcome.NotReadable(detail = problem.detail)
        }
    }

    fun of(answer: AssistantAnswer): AnswerOutcome {
        return when (answer) {
            is AssistantAnswer.Text -> AnswerOutcome.Answered(text = answer.text, model = answer.model)
            is AssistantAnswer.Failed -> AnswerOutcome.Failed(reason = answer.reason, detail = answer.detail)
            is AssistantAnswer.Tags -> AnswerOutcome.Failed(
                reason = ClassificationFailure.INVALID_RESPONSE,
                detail = "Tags where text was expected",
            )
        }
    }
}
