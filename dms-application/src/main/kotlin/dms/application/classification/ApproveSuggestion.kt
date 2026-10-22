package dms.application.classification

import dms.application.classification.model.ApprovedCategory
import dms.application.classification.model.SuggestionApproval
import dms.application.common.ActivityRecorder
import dms.application.common.LibrarySnapshot
import dms.application.common.OperationOutcome
import dms.application.document.MoveDocument
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.ClassificationRecord
import dms.domain.document.ClassificationSource
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import java.util.Locale

/**
 * Applies the approved parts of a suggestion: renames the file in every folder if a new name was
 * approved, then writes metadata, category and tags to the catalog in one change and records that
 * the AI classification was confirmed.
 */
class ApproveSuggestion(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val moveDocument: MoveDocument,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(documentId: DocumentId, approval: SuggestionApproval): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        val suggestion = tasks.suggestions().firstOrNull { it.documentId == documentId }
            ?: return OperationOutcome.NotFound
        // The ids of the categories a path may create, fixed here so the check and the commit agree.
        val newCategoryIds = newCategoryIdsFor(category = approval.category)
        // Check the catalog part first, so a refusal does not leave a renamed file behind.
        val check = classify(
            catalog = library.catalog,
            suggestion = suggestion,
            approval = approval,
            newCategoryIds = newCategoryIds,
        )
        if (check is CatalogUpdate.Rejected) {
            return OperationOutcome.Rejected(reason = check.reason)
        }
        val renamed = rename(library = library, document = document, fileName = approval.fileName)
        if (renamed !is OperationOutcome.Done) {
            return renamed
        }
        val outcome = store.commit { catalog ->
            classify(catalog = catalog, suggestion = suggestion, approval = approval, newCategoryIds = newCategoryIds)
        }.withPendingFolders(folders = renamed.pendingFolders)
        if (outcome is OperationOutcome.Done) {
            tasks.removeSuggestion(documentId = documentId)
            val path = outcome.catalog.documents[documentId]?.path ?: document.path
            activity.record(kind = ActivityKind.SUGGESTION_APPROVED, subject = path.value)
        }
        return outcome
    }

    private fun newCategoryIdsFor(category: ApprovedCategory?): List<CategoryId> {
        val levels = (category as? ApprovedCategory.Path)?.names?.size ?: 0
        return List(size = levels) { CategoryId(value = ids.newId()) }
    }

    private suspend fun rename(library: LibrarySnapshot, document: Document, fileName: String?): OperationOutcome {
        if (fileName == null || fileName == document.path.fileName) {
            return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
        }
        val parent = document.path.parent ?: RelativePath.ROOT
        return moveDocument(documentId = document.id, newPath = parent.resolve(name = fileName))
    }

    private fun classify(
        catalog: Catalog,
        suggestion: ClassificationSuggestion,
        approval: SuggestionApproval,
        newCategoryIds: List<CategoryId>,
    ): CatalogUpdate {
        val documentId = suggestion.documentId
        val document = catalog.documents[documentId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        val metadata = DocumentMetadata(
            title = approval.title ?: document.metadata.title,
            documentDate = approval.documentDate ?: document.metadata.documentDate,
            correspondent = approval.correspondent ?: document.metadata.correspondent,
        )
        val record = ClassificationRecord(
            source = ClassificationSource.AI,
            model = suggestion.model,
            confirmedAt = clock.now(),
        )
        return withCategory(
            catalog = catalog,
            documentId = documentId,
            category = approval.category,
            newIds = newCategoryIds,
        )
            .andThen { it.updateMetadata(documentId = documentId, metadata = metadata) }
            .andThen { it.setTags(documentId = documentId, newTags = document.tags + approval.tags) }
            .andThen { it.recordClassification(documentId = documentId, record = record) }
    }

    private fun withCategory(
        catalog: Catalog,
        documentId: DocumentId,
        category: ApprovedCategory?,
        newIds: List<CategoryId>,
    ): CatalogUpdate {
        return when (category) {
            null -> CatalogUpdate.Applied(catalog = catalog)
            is ApprovedCategory.Existing -> catalog.assignCategory(
                documentId = documentId,
                categoryId = category.categoryId,
            )

            is ApprovedCategory.Path -> withPath(
                catalog = catalog,
                documentId = documentId,
                names = category.names,
                newIds = newIds,
            )
        }
    }

    /**
     * Walks [names] from the top level down: a level whose name already exists at that place, ignoring
     * case, is reused, every missing one is created, and the document gets the last level.
     */
    private fun withPath(
        catalog: Catalog,
        documentId: DocumentId,
        names: List<String>,
        newIds: List<CategoryId>,
    ): CatalogUpdate {
        var current = catalog
        var parentId: CategoryId? = null
        names.zip(other = newIds).forEach { (name, newId) ->
            val existing = childNamed(catalog = current, parentId = parentId, name = name)
            if (existing != null) {
                parentId = existing.id
            } else {
                val created = Category(id = newId, name = name.trim(), parentId = parentId)
                val added = current.addCategory(category = created)
                current = (added as? CatalogUpdate.Applied)?.catalog ?: return added
                parentId = newId
            }
        }
        return current.assignCategory(documentId = documentId, categoryId = checkNotNull(value = parentId))
    }

    private fun childNamed(catalog: Catalog, parentId: CategoryId?, name: String): Category? {
        val key = name.trim().lowercase(Locale.ROOT)
        return catalog.categories.values.firstOrNull {
            it.parentId == parentId && it.name.trim().lowercase(Locale.ROOT) == key
        }
    }
}
