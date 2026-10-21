package dms.ui.dashboard

import dms.application.classification.model.ClassificationProgress
import dms.application.classification.model.SuggestOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.DocumentId
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.OperationProgress
import dms.ui.session.model.UiMessage
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.takeWhile

/**
 * The AI's runs over the whole library: classify every document without a classification, or tag
 * every document without tags. One document after the other, with progress, until the user cancels;
 * a run stops at once when the AI server does not answer. Every result waits under Open tasks.
 */
class AiBatches(
    private val services: AppServices,
    private val session: LibrarySession,
) {

    fun classifyAll() {
        run(step = "Classifying with AI", batch = { services.ai.classifyAllNew() }) { count ->
            "The AI classified $count documents; review them under Open tasks"
        }
    }

    fun tagAll() {
        run(step = "Tagging with AI", batch = { services.ai.tagAllUntagged() }) { count ->
            "The AI suggested tags for $count documents; review them under Open tasks"
        }
    }

    private fun run(step: String, batch: () -> Flow<ClassificationProgress>, done: (Int) -> String) {
        session.perform(label = step, cancellable = true) {
            var suggested = 0
            var failure: SuggestOutcome.Failed? = null
            session.progress(progress = OperationProgress(step = step))
            batch()
                .onEach { progress ->
                    val name = nameOf(documentId = progress.documentId)
                    val current = OperationProgress(step = step, done = progress.done, total = progress.total)
                    session.progress(progress = current.copy(detail = name))
                    when (val outcome = progress.outcome) {
                        is SuggestOutcome.Suggested -> if (outcome.suggestion.hasValues()) {
                            suggested++
                            session.log.info(text = "AI suggestion for $name")
                        } else {
                            session.log.info(text = "The AI found nothing new for $name")
                        }

                        else -> session.log.error(text = AiMessages.of(outcome = outcome, name = name).text)
                    }
                    failure = (progress.outcome as? SuggestOutcome.Failed)
                        ?.takeIf { it.reason == ClassificationFailure.PROVIDER_UNAVAILABLE }
                }
                .takeWhile { failure == null }
                .collect()
            val stopped = failure
            session.report(
                message = if (stopped != null) {
                    UiMessage(text = AiMessages.failure(reason = stopped.reason), isError = true)
                } else {
                    UiMessage(text = done(suggested))
                },
            )
        }
    }

    private fun ClassificationSuggestion.hasValues(): Boolean {
        return tags.isNotEmpty() || listOf(title, category, documentDate, correspondent, fileName).any { it != null }
    }

    private fun nameOf(documentId: DocumentId): String {
        return services.library.catalog.value?.documents?.get(documentId)?.path?.value ?: documentId.value
    }
}
