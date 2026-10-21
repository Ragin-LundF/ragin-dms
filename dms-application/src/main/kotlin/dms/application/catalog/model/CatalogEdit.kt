package dms.application.catalog.model

import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName

/** A change to the categorization in the catalog that touches no file. */
sealed interface CatalogEdit {

    data class UpdateMetadata(val documentId: DocumentId, val metadata: DocumentMetadata) : CatalogEdit

    /** Sets the category of a document, or clears it with `null`. */
    data class AssignCategory(val documentId: DocumentId, val categoryId: CategoryId?) : CatalogEdit

    data class AddTags(val documentId: DocumentId, val tags: Set<TagName>) : CatalogEdit

    data class RemoveTags(val documentId: DocumentId, val tags: Set<TagName>) : CatalogEdit

    /** Creates a category; its id is generated. */
    data class AddCategory(val name: String, val parentId: CategoryId?) : CatalogEdit

    data class RenameCategory(val categoryId: CategoryId, val name: String) : CatalogEdit

    data class MoveCategory(val categoryId: CategoryId, val parentId: CategoryId?) : CatalogEdit

    data class RemoveCategory(val categoryId: CategoryId) : CatalogEdit

    /** Renames a tag everywhere; renaming to an existing tag merges the two. */
    data class RenameTag(val from: TagName, val to: TagName) : CatalogEdit

    data class RemoveTag(val tag: TagName) : CatalogEdit
}
