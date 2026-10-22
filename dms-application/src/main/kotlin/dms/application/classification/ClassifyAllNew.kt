package dms.application.classification

import dms.application.classification.model.ClassificationProgressModel
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Suggests a classification for every document the AI can read that has neither a confirmed
 * classification nor a pending suggestion, one after the other, optionally only below one folder such
 * as the inbox. Cancelling the collection stops it.
 */
class ClassifyAllNew(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val suggest: SuggestClassification,
) {

    operator fun invoke(scope: RelativePathModel = RelativePathModel.ROOT): Flow<ClassificationProgressModel> {
        return flow {
            val library = store.current() ?: return@flow
            val pending = tasks.suggestions().map { it.documentId }.toSet()
            val candidates = library.catalog.documents.values
                .filter { it.classification == null && it.kind.readableByAi && it.id !in pending }
                .filter { it.path.isInside(folder = scope) }
                .sortedBy { it.path.value }
            candidates.forEachIndexed { index, document ->
                val progress = ClassificationProgressModel(
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
