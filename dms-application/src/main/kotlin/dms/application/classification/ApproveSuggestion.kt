package dms.application.classification

import dms.application.classification.model.ApprovedCategory
import dms.application.classification.model.SuggestionApprovalModel
import dms.application.common.ActivityRecorder
import dms.application.common.DocumentFiling
import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import java.util.Locale

/**
 * Applies the approved parts of a suggestion: renames the file in every folder if a new name was
 * approved and files it into the approved target folder, in one move, then writes metadata, category
 * and tags to the catalog in one change and records that the AI classification was confirmed.
 */
class ApproveSuggestion(
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val filing: DocumentFiling,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(documentId: DocumentIdModel, approval: SuggestionApprovalModel): OperationOutcome {
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
        val renamed = move(library = library, document = document, approval = approval)
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

    private fun newCategoryIdsFor(category: ApprovedCategory?): List<CategoryIdModel> {
        val levels = (category as? ApprovedCategory.Path)?.names?.size ?: 0
        return List(size = levels) { CategoryIdModel(value = ids.newId()) }
    }

    private suspend fun move(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        approval: SuggestionApprovalModel,
    ): OperationOutcome {
        val parent = document.path.parent ?: RelativePathModel.ROOT
        val folder = approval.targetFolder ?: parent
        val fileName = approval.fileName ?: document.path.fileName
        if (folder == parent && fileName == document.path.fileName) {
            return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
        }
        return filing.file(documentId = document.id, folder = folder, fileName = fileName)
    }

    private fun classify(
        catalog: CatalogModel,
        suggestion: ClassificationSuggestionModel,
        approval: SuggestionApprovalModel,
        newCategoryIds: List<CategoryIdModel>,
    ): CatalogUpdate {
        val documentId = suggestion.documentId
        val document = catalog.documents[documentId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        val metadata = DocumentMetadataModel(
            title = approval.title ?: document.metadata.title,
            documentDate = approval.documentDate ?: document.metadata.documentDate,
            correspondent = approval.correspondent ?: document.metadata.correspondent,
        )
        val record = ClassificationRecordModel(
            source = if (suggestion.byRules) ClassificationSource.RULE else ClassificationSource.AI,
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
            .andThen { withFields(catalog = it, document = document, approval = approval) }
            .andThen { it.recordClassification(documentId = documentId, record = record) }
    }

    private fun withFields(
        catalog: CatalogModel,
        document: DocumentModel,
        approval: SuggestionApprovalModel,
    ): CatalogUpdate {
        if (approval.fields.isEmpty()) {
            return CatalogUpdate.Applied(catalog = catalog)
        }
        return catalog.setFields(documentId = document.id, fields = document.fields + approval.fields)
    }

    private fun withCategory(
        catalog: CatalogModel,
        documentId: DocumentIdModel,
        category: ApprovedCategory?,
        newIds: List<CategoryIdModel>,
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
        catalog: CatalogModel,
        documentId: DocumentIdModel,
        names: List<String>,
        newIds: List<CategoryIdModel>,
    ): CatalogUpdate {
        var current = catalog
        var parentId: CategoryIdModel? = null
        names.zip(other = newIds).forEach { (name, newId) ->
            val existing = childNamed(catalog = current, parentId = parentId, name = name)
            if (existing != null) {
                parentId = existing.id
            } else {
                val created = CategoryModel(id = newId, name = name.trim(), parentId = parentId)
                val added = current.addCategory(category = created)
                current = (added as? CatalogUpdate.Applied)?.catalog ?: return added
                parentId = newId
            }
        }
        return current.assignCategory(documentId = documentId, categoryId = checkNotNull(value = parentId))
    }

    private fun childNamed(catalog: CatalogModel, parentId: CategoryIdModel?, name: String): CategoryModel? {
        val key = name.trim().lowercase(Locale.ROOT)
        return catalog.categories.values.firstOrNull {
            it.parentId == parentId && it.name.trim().lowercase(Locale.ROOT) == key
        }
    }
}
