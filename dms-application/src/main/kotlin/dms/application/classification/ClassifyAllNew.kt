package dms.application.classification

import dms.application.classification.model.ClassificationProgress
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Suggests a classification for every document the AI can read that has neither a confirmed
 * classification nor a pending suggestion, one after the other. Cancelling the collection stops it.
 */
class ClassifyAllNew(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val suggest: SuggestClassification,
) {

    operator fun invoke(): Flow<ClassificationProgress> {
        return flow {
            val library = store.current() ?: return@flow
            val pending = tasks.suggestions().map { it.documentId }.toSet()
            val candidates = library.catalog.documents.values
                .filter { it.classification == null && it.kind.readableByAi && it.id !in pending }
                .sortedBy { it.path.value }
            candidates.forEachIndexed { index, document ->
                val progress = ClassificationProgress(
                    done = index + 1,
                    total = candidates.size,
                    documentId = document.id,
                    outcome = suggest(documentId = document.id),
                )
                emit(value = progress)
            }
        }
    }
}
