package dms.application.catalog.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/** A change to the categorization in the catalog that touches no file. */
sealed interface CatalogEdit {

    data class UpdateMetadata(val documentId: DocumentIdModel, val metadata: DocumentMetadataModel) : CatalogEdit

    /** Sets the category of a document, or clears it with `null`. */
    data class AssignCategory(val documentId: DocumentIdModel, val categoryId: CategoryIdModel?) : CatalogEdit

    data class AddTags(val documentId: DocumentIdModel, val tags: Set<TagNameModel>) : CatalogEdit

    data class RemoveTags(val documentId: DocumentIdModel, val tags: Set<TagNameModel>) : CatalogEdit

    /** Creates a category; its id is generated. */
    data class AddCategory(val name: String, val parentId: CategoryIdModel?) : CatalogEdit

    data class RenameCategory(val categoryId: CategoryIdModel, val name: String) : CatalogEdit

    data class MoveCategory(val categoryId: CategoryIdModel, val parentId: CategoryIdModel?) : CatalogEdit

    data class RemoveCategory(val categoryId: CategoryIdModel) : CatalogEdit

    /** Renames a tag everywhere; renaming to an existing tag merges the two. */
    data class RenameTag(val from: TagNameModel, val to: TagNameModel) : CatalogEdit

    data class RemoveTag(val tag: TagNameModel) : CatalogEdit
}
