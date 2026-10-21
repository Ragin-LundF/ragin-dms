package dms.application.classification

import dms.application.classification.model.SuggestOutcome
import dms.application.common.LibrarySnapshot
import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
import dms.application.port.state.TaskStore
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.FolderOperation
import dms.application.port.system.Clock
import dms.application.sync.Replicator
import dms.domain.catalog.Catalog
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.document.Document
import dms.domain.document.DocumentId

/**
 * Lets the AI suggest a classification for one document. The suggestion is stored as a task for the
 * user's review; nothing in the catalog or on disk changes, except the document's text in
 * `.dms-text/`, which search uses.
 */
class SuggestClassification(
    private val store: CatalogStore,
    private val reader: DocumentContentReader,
    private val classifier: DocumentClassifier,
    private val tasks: TaskStore,
    private val texts: TextStore,
    private val replicator: Replicator,
    private val clock: Clock,
) {

    suspend operator fun invoke(documentId: DocumentId): SuggestOutcome {
        val library = store.current() ?: return SuggestOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return SuggestOutcome.NotFound
        if (!document.kind.readableByAi) {
            return SuggestOutcome.NotReadable(detail = null)
        }
        val content = reader.read(folder = library.folders.primary, document = document)
        if (content is DocumentContent.NotReadable) {
            return SuggestOutcome.NotReadable(detail = content.reason)
        }
        val request = request(catalog = library.catalog, document = document, content = content)
        return when (val result = classifier.suggest(request = request)) {
            is ClassificationResult.Failed -> SuggestOutcome.Failed(reason = result.reason, detail = result.detail)
            is ClassificationResult.Suggested -> {
                val suggestion = normalized(
                    suggestion = result.suggestion,
                    catalog = library.catalog,
                    document = document,
                )
                tasks.saveSuggestion(suggestion = suggestion)
                storeText(library = library, documentId = documentId, content = content, suggestion = suggestion)
                SuggestOutcome.Suggested(suggestion = suggestion)
            }
        }
    }

    private fun request(catalog: Catalog, document: Document, content: DocumentContent): ClassificationRequest {
        val others = catalog.documents.values.filter { it.id != document.id }
        return ClassificationRequest(
            documentId = document.id,
            currentFileName = document.path.fileName,
            content = content,
            categories = catalog.categories.values.sortedBy { it.name },
            tags = catalog.tags,
            folders = others.mapNotNull { it.path.parent }.filterNot { it.isRoot }.distinct().sortedBy { it.value },
            fileNameExamples = others.sortedByDescending { it.addedAt }
                .take(n = FILE_NAME_EXAMPLES)
                .map { it.path.fileName },
            today = clock.today(),
        )
    }

    /** Ties the suggestion to the document and checks it against the catalog as it is now. */
    private fun normalized(
        suggestion: ClassificationSuggestion,
        catalog: Catalog,
        document: Document,
    ): ClassificationSuggestion {
        val category = suggestion.category
        val knownCategory = category !is SuggestedCategory.Existing || category.categoryId in catalog.categories
        return suggestion.copy(
            documentId = document.id,
            category = if (knownCategory) category else null,
            tags = suggestion.tags.map { it.copy(isNew = it.name !in catalog.tags) },
        )
    }

    /** The text is derived data: a failure to store it does not fail the suggestion. */
    private suspend fun storeText(
        library: LibrarySnapshot,
        documentId: DocumentId,
        content: DocumentContent,
        suggestion: ClassificationSuggestion,
    ) {
        val text = (content as? DocumentContent.Text)?.text ?: suggestion.transcription
        if (text.isNullOrBlank()) {
            return
        }
        replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> texts.write(folder = primary, documentId = documentId, text = text) },
            followUp = FolderOperation.CopyText(documentId = documentId),
        )
    }

    private companion object {
        const val FILE_NAME_EXAMPLES = 20
    }
}
