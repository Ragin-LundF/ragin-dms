package dms.domain.catalog

import dms.domain.document.ClassificationRecord
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import java.util.Locale

/**
 * The aggregate root: every document of a library with its categorization, plus the categories and
 * tags in use.
 *
 * A catalog is immutable. Every change goes through one of its functions, which returns either a new
 * catalog with the next [revision] or the reason the change was refused. The invariants checked in
 * `init` therefore hold for every catalog that exists.
 */
data class Catalog(
    val libraryId: LibraryId,
    val revision: CatalogRevision,
    val documents: Map<DocumentId, Document>,
    val categories: Map<CategoryId, Category>,
    val tags: Set<TagName>,
) {

    init {
        require(value = documents.all { (id, document) -> id == document.id }) { "Document keys must match ids" }
        require(value = documents.values.distinctBy { it.path }.size == documents.size) {
            "Two documents share a path"
        }
        require(value = categories.all { (id, category) -> id == category.id }) { "Category keys must match ids" }
        require(value = categories.values.all { it.parentId == null || it.parentId in categories }) {
            "A category refers to an unknown parent"
        }
        require(value = categories.keys.none { isInCycle(categoryId = it, parents = categories) }) {
            "The category tree contains a cycle"
        }
        require(value = documents.values.all { it.categoryId == null || it.categoryId in categories }) {
            "A document refers to an unknown category"
        }
        require(value = documents.values.all { tags.containsAll(elements = it.tags) }) {
            "A document uses a tag the catalog does not know"
        }
    }

    // --- Queries

    fun findByPath(path: RelativePath): Document? {
        return documents.values.firstOrNull { it.path == path }
    }

    fun findByContentHash(contentHash: ContentHash): List<Document> {
        return documents.values.filter { it.contentHash == contentHash }
    }

    /** Documents directly or indirectly below [folder]. */
    fun documentsInside(folder: RelativePath): List<Document> {
        return documents.values.filter { it.path.isInside(folder = folder) }
    }

    // --- Documents

    fun addDocument(document: Document): CatalogUpdate {
        if (document.id in documents) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_ALREADY_EXISTS)
        }
        if (findByPath(path = document.path) != null) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
        }
        if (document.categoryId != null && document.categoryId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        val canonical = document.copy(tags = canonicalTags(requested = document.tags))
        return applied(documents = documents + (document.id to canonical), tags = tags + canonical.tags)
    }

    /** Removes the catalog entry only; what happens to the file is the caller's decision. */
    fun removeDocument(documentId: DocumentId): CatalogUpdate {
        if (documentId !in documents) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        }
        return applied(documents = documents - documentId)
    }

    /** Moves or renames one document. */
    fun moveDocument(documentId: DocumentId, newPath: RelativePath): CatalogUpdate {
        val owner = findByPath(path = newPath)
        if (owner != null && owner.id != documentId) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
        }
        return updateDocument(documentId = documentId) { it.copy(path = newPath) }
    }

    /** Moves or renames a folder: every document below [from] ends up below [to]. */
    fun moveFolder(from: RelativePath, to: RelativePath): CatalogUpdate {
        if (from.isRoot || from == to || to.isInside(folder = from)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE)
        }
        val (moving, staying) = documents.values.partition { it.path.isInside(folder = from) }
        val moved = moving.map { it.copy(path = it.path.relocate(from = from, to = to)) }
        val occupiedPaths = staying.map { it.path }.toSet()
        if (moved.any { it.path in occupiedPaths }) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
        }
        return applied(documents = documents + moved.associateBy { it.id })
    }

    /** Records new content of a file that was changed on disk. */
    fun updateContent(documentId: DocumentId, contentHash: ContentHash, size: Long): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(contentHash = contentHash, size = size) }
    }

    fun updateMetadata(documentId: DocumentId, metadata: DocumentMetadata): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(metadata = metadata) }
    }

    /** Sets or, with `null`, clears the category of a document. */
    fun assignCategory(documentId: DocumentId, categoryId: CategoryId?): CatalogUpdate {
        if (categoryId != null && categoryId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        return updateDocument(documentId = documentId) { it.copy(categoryId = categoryId) }
    }

    /** Replaces the tags of a document. Tags the catalog does not know yet are added to it. */
    fun setTags(documentId: DocumentId, newTags: Set<TagName>): CatalogUpdate {
        val canonical = canonicalTags(requested = newTags)
        return updateDocument(documentId = documentId, addedTags = canonical) { it.copy(tags = canonical) }
    }

    fun recordClassification(documentId: DocumentId, record: ClassificationRecord): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(classification = record) }
    }

    // --- Categories

    fun addCategory(category: Category): CatalogUpdate {
        if (category.id in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_ALREADY_EXISTS)
        }
        if (category.parentId != null && category.parentId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        if (isNameTaken(name = category.name, parentId = category.parentId, except = null)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NAME_TAKEN)
        }
        return applied(categories = categories + (category.id to category))
    }

    fun renameCategory(categoryId: CategoryId, name: String): CatalogUpdate {
        val category = categories[categoryId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        if (isNameTaken(name = name, parentId = category.parentId, except = categoryId)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NAME_TAKEN)
        }
        return applied(categories = categories + (categoryId to category.copy(name = name)))
    }

    /** Moves a category below [parentId], or to the top level with `null`. */
    fun moveCategory(categoryId: CategoryId, parentId: CategoryId?): CatalogUpdate {
        val category = categories[categoryId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        if (parentId != null && parentId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        if (parentId == categoryId) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_CYCLE)
        }
        val moved = categories + (categoryId to category.copy(parentId = parentId))
        if (isInCycle(categoryId = categoryId, parents = moved)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_CYCLE)
        }
        if (isNameTaken(name = category.name, parentId = parentId, except = categoryId)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NAME_TAKEN)
        }
        return applied(categories = moved)
    }

    /** Removes an unused category without children. */
    fun removeCategory(categoryId: CategoryId): CatalogUpdate {
        if (categoryId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        if (documents.values.any { it.categoryId == categoryId }) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_IN_USE)
        }
        if (categories.values.any { it.parentId == categoryId }) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_HAS_CHILDREN)
        }
        return applied(categories = categories - categoryId)
    }

    // --- Tags

    /**
     * Renames [from] to [to] everywhere. If [to] already exists, the two tags are merged. Renaming to
     * a different casing of the same tag changes how it is displayed.
     */
    fun renameTag(from: TagName, to: TagName): CatalogUpdate {
        if (from !in tags) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.TAG_NOT_FOUND)
        }
        val remaining = tags - from
        // Merging into an existing tag keeps that tag's spelling.
        val target = remaining.firstOrNull { it == to } ?: to
        val renamed = documents.mapValues { (_, document) ->
            if (from in document.tags) document.copy(tags = document.tags - from + target) else document
        }
        return applied(documents = renamed, tags = remaining + target)
    }

    /** Removes a tag from the catalog and from every document. */
    fun removeTag(tag: TagName): CatalogUpdate {
        if (tag !in tags) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.TAG_NOT_FOUND)
        }
        val untagged = documents.mapValues { (_, document) -> document.copy(tags = document.tags - tag) }
        return applied(documents = untagged, tags = tags - tag)
    }

    // --- Helpers

    private fun updateDocument(
        documentId: DocumentId,
        addedTags: Set<TagName> = emptySet(),
        change: (Document) -> Document,
    ): CatalogUpdate {
        val document = documents[documentId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        return applied(documents = documents + (documentId to change(document)), tags = tags + addedTags)
    }

    private fun applied(
        documents: Map<DocumentId, Document> = this.documents,
        categories: Map<CategoryId, Category> = this.categories,
        tags: Set<TagName> = this.tags,
    ): CatalogUpdate {
        val catalog = copy(revision = revision.next(), documents = documents, categories = categories, tags = tags)
        return CatalogUpdate.Applied(catalog = catalog)
    }

    /** Uses the spelling the catalog already knows for tags that differ only in casing. */
    private fun canonicalTags(requested: Set<TagName>): Set<TagName> {
        return requested.map { tag -> tags.firstOrNull { it == tag } ?: tag }.toSet()
    }

    private fun isNameTaken(name: String, parentId: CategoryId?, except: CategoryId?): Boolean {
        val key = name.trim().lowercase(Locale.ROOT)
        return categories.values.any {
            it.id != except && it.parentId == parentId && it.name.trim().lowercase(Locale.ROOT) == key
        }
    }

    companion object {

        fun empty(libraryId: LibraryId): Catalog {
            return Catalog(
                libraryId = libraryId,
                revision = CatalogRevision.INITIAL,
                documents = emptyMap(),
                categories = emptyMap(),
                tags = emptySet(),
            )
        }

        /** `true` if following the parents of [categoryId] leads back to it. */
        private fun isInCycle(categoryId: CategoryId, parents: Map<CategoryId, Category>): Boolean {
            val visited = mutableSetOf<CategoryId>()
            var current = parents[categoryId]?.parentId
            while (current != null) {
                if (current == categoryId || !visited.add(element = current)) {
                    return true
                }
                current = parents[current]?.parentId
            }
            return false
        }
    }
}
