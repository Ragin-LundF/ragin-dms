package dms.application.assistant

import dms.application.classification.model.ClassificationProgress
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Lets the AI suggest tags for every document it can read that has no tags and no open suggestion,
 * one after the other. Each result is emitted, so the UI can show progress and stop the batch.
 */
class TagAllUntagged(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val suggestTags: SuggestTags,
) {

    operator fun invoke(): Flow<ClassificationProgress> {
        return flow {
            val library = store.current() ?: return@flow
            val pending = tasks.suggestions().map { it.documentId }.toSet()
            val candidates = library.catalog.documents.values
                .filter { it.tags.isEmpty() && it.kind.readableByAi && it.id !in pending }
                .sortedBy { it.path.value }
            candidates.forEachIndexed { index, document ->
                val progress = ClassificationProgress(
                    done = index + 1,
                    total = candidates.size,
                    documentId = document.id,
                    outcome = suggestTags(documentId = document.id),
                )
                emit(value = progress)
            }
        }
    }
}
