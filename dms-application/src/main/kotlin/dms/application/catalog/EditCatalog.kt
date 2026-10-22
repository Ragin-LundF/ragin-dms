package dms.application.catalog

import dms.application.catalog.model.CatalogEdit
import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.IdGenerator
import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel

/**
 * Applies one manual change to metadata, categories or tags and saves the catalog to every library
 * folder. Manual categorization works without any AI.
 */
class EditCatalog(
    private val store: CatalogStore,
    private val activity: ActivityRecorder,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(edit: CatalogEdit): OperationOutcome {
        val outcome = store.commit { catalog -> apply(catalog = catalog, edit = edit) }
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.CATALOG_EDITED, subject = edit::class.simpleName.orEmpty())
        }
        return outcome
    }

    private fun apply(catalog: CatalogModel, edit: CatalogEdit): CatalogUpdate {
        return when (edit) {
            is CatalogEdit.UpdateMetadata -> catalog.updateMetadata(
                documentId = edit.documentId,
                metadata = edit.metadata,
            )

            is CatalogEdit.AssignCategory -> catalog.assignCategory(
                documentId = edit.documentId,
                categoryId = edit.categoryId,
            )

            is CatalogEdit.AddTags -> changeTags(catalog = catalog, documentId = edit.documentId) { it + edit.tags }
            is CatalogEdit.RemoveTags -> changeTags(catalog = catalog, documentId = edit.documentId) { it - edit.tags }
            is CatalogEdit.AddCategory -> catalog.addCategory(
                category = CategoryModel(
                    id = CategoryIdModel(value = ids.newId()),
                    name = edit.name,
                    parentId = edit.parentId,
                ),
            )

            is CatalogEdit.RenameCategory -> catalog.renameCategory(categoryId = edit.categoryId, name = edit.name)
            is CatalogEdit.MoveCategory -> catalog.moveCategory(categoryId = edit.categoryId, parentId = edit.parentId)
            is CatalogEdit.RemoveCategory -> catalog.removeCategory(categoryId = edit.categoryId)
            is CatalogEdit.RenameTag -> catalog.renameTag(from = edit.from, to = edit.to)
            is CatalogEdit.RemoveTag -> catalog.removeTag(tag = edit.tag)
        }
    }

    private fun changeTags(
        catalog: CatalogModel,
        documentId: DocumentIdModel,
        change: (Set<TagNameModel>) -> Set<TagNameModel>,
    ): CatalogUpdate {
        val document = catalog.documents[documentId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        return catalog.setTags(documentId = documentId, newTags = change(document.tags))
    }
}
