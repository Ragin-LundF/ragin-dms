package dms.domain.catalog.model

import dms.domain.catalog.CatalogRejection
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.link.model.DocumentLinkModel
import dms.domain.retention.model.RetentionModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.util.Locale

/**
 * The aggregate root: every document of a library with its categorization, plus the categories and
 * tags in use, the [links] between documents and the library-wide [settings].
 *
 * A catalog is immutable. Every change goes through one of its functions, which returns either a new
 * catalog with the next [revision] or the reason the change was refused. The invariants checked in
 * `init` therefore hold for every catalog that exists.
 */
@Suppress("TooManyFunctions") // The aggregate root: every change to the catalog must go through it.
data class CatalogModel(
    val libraryId: LibraryIdModel,
    val revision: CatalogRevisionModel,
    val documents: Map<DocumentIdModel, DocumentModel>,
    val categories: Map<CategoryIdModel, CategoryModel>,
    val tags: Set<TagNameModel>,
    val links: Set<DocumentLinkModel> = emptySet(),
    val settings: CatalogSettingsModel = CatalogSettingsModel.EMPTY,
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
        require(value = links.all { it.first in documents && it.second in documents }) {
            "A link refers to an unknown document"
        }
        require(value = links.distinctBy { it.first to it.second }.size == links.size) {
            "Two links connect the same documents"
        }
    }

    // --- Queries

    fun findByPath(path: RelativePathModel): DocumentModel? {
        return documents.values.firstOrNull { it.path == path }
    }

    fun findByContentHash(contentHash: ContentHashModel): List<DocumentModel> {
        return documents.values.filter { it.contentHash == contentHash }
    }

    /** Documents directly or indirectly below [folder]. */
    fun documentsInside(folder: RelativePathModel): List<DocumentModel> {
        return documents.values.filter { it.path.isInside(folder = folder) }
    }

    /** The links of [documentId], whatever end it is on. */
    fun linksOf(documentId: DocumentIdModel): List<DocumentLinkModel> {
        return links.filter { it.touches(documentId = documentId) }
    }

    // --- Documents

    fun addDocument(document: DocumentModel): CatalogUpdate {
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

    /** Removes the catalog entry and its links; what happens to the file is the caller's decision. */
    fun removeDocument(documentId: DocumentIdModel): CatalogUpdate {
        if (documentId !in documents) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        }
        val remainingLinks = links.filterNot { it.touches(documentId = documentId) }.toSet()
        return applied(documents = documents - documentId, links = remainingLinks)
    }

    /** Moves or renames one document. */
    fun moveDocument(documentId: DocumentIdModel, newPath: RelativePathModel): CatalogUpdate {
        val owner = findByPath(path = newPath)
        if (owner != null && owner.id != documentId) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
        }
        return updateDocument(documentId = documentId) { it.copy(path = newPath) }
    }

    /**
     * Moves many documents at once, in one revision: every document of [moves] gets its new path. Paths
     * may be swapped among the moved documents; no path may end up used twice.
     */
    fun moveDocuments(moves: Map<DocumentIdModel, RelativePathModel>): CatalogUpdate {
        if (moves.keys.any { it !in documents }) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        }
        val moved = documents.mapValues { (id, document) -> moves[id]?.let { document.copy(path = it) } ?: document }
        if (moved.values.distinctBy { it.path }.size != moved.size) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED)
        }
        return applied(documents = moved)
    }

    /** Moves or renames a folder: every document below [from] ends up below [to]. */
    fun moveFolder(from: RelativePathModel, to: RelativePathModel): CatalogUpdate {
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
    fun updateContent(documentId: DocumentIdModel, contentHash: ContentHashModel, size: Long): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(contentHash = contentHash, size = size) }
    }

    fun updateMetadata(documentId: DocumentIdModel, metadata: DocumentMetadataModel): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(metadata = metadata) }
    }

    /** Sets or, with `null`, clears the category of a document. */
    fun assignCategory(documentId: DocumentIdModel, categoryId: CategoryIdModel?): CatalogUpdate {
        if (categoryId != null && categoryId !in categories) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        }
        return updateDocument(documentId = documentId) { it.copy(categoryId = categoryId) }
    }

    /** Replaces the tags of a document. Tags the catalog does not know yet are added to it. */
    fun setTags(documentId: DocumentIdModel, newTags: Set<TagNameModel>): CatalogUpdate {
        val canonical = canonicalTags(requested = newTags)
        return updateDocument(documentId = documentId, addedTags = canonical) { it.copy(tags = canonical) }
    }

    fun recordClassification(documentId: DocumentIdModel, record: ClassificationRecordModel): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(classification = record) }
    }

    /**
     * Replaces the custom field values of a document. Every key needs a definition in [settings], and
     * every value the type of its definition.
     */
    fun setFields(documentId: DocumentIdModel, fields: Map<FieldKeyModel, FieldValue>): CatalogUpdate {
        fields.forEach { (key, value) ->
            val definition = settings.field(key = key)
                ?: return CatalogUpdate.Rejected(reason = CatalogRejection.FIELD_NOT_DEFINED)
            if (definition.type != value.type) {
                return CatalogUpdate.Rejected(reason = CatalogRejection.FIELD_TYPE_MISMATCH)
            }
        }
        return updateDocument(documentId = documentId) { it.copy(fields = fields) }
    }

    /** Sets the note of a document; a blank note removes it. */
    fun setNote(documentId: DocumentIdModel, note: String?): CatalogUpdate {
        val cleaned = note?.trimEnd()?.takeIf { it.isNotBlank() }
        return updateDocument(documentId = documentId) { it.copy(note = cleaned) }
    }

    /** Sets the opt-in retention of a document; `null` or an empty retention removes it. */
    fun setRetention(documentId: DocumentIdModel, retention: RetentionModel?): CatalogUpdate {
        return updateDocument(documentId = documentId) { it.copy(retention = retention?.takeUnless { it.isEmpty }) }
    }

    // --- Links

    /** Links two documents; an existing link between them takes the new label. */
    fun link(link: DocumentLinkModel): CatalogUpdate {
        if (link.first !in documents || link.second !in documents) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        }
        return applied(links = links.filterNot { it.samePair(other = link) }.toSet() + link)
    }

    fun unlink(first: DocumentIdModel, second: DocumentIdModel): CatalogUpdate {
        val remaining = links.filterNot { it.touches(documentId = first) && it.otherThan(documentId = first) == second }
        if (remaining.size == links.size) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.LINK_NOT_FOUND)
        }
        return applied(links = remaining.toSet())
    }

    // --- Settings

    /** Replaces the library-wide settings: field definitions, saved searches, rules and layout. */
    fun changeSettings(settings: CatalogSettingsModel): CatalogUpdate {
        return applied(settings = settings)
    }

    // --- Categories

    fun addCategory(category: CategoryModel): CatalogUpdate {
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

    fun renameCategory(categoryId: CategoryIdModel, name: String): CatalogUpdate {
        val category = categories[categoryId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        if (isNameTaken(name = name, parentId = category.parentId, except = categoryId)) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NAME_TAKEN)
        }
        return applied(categories = categories + (categoryId to category.copy(name = name)))
    }

    /** Moves a category below [parentId], or to the top level with `null`. */
    fun moveCategory(categoryId: CategoryIdModel, parentId: CategoryIdModel?): CatalogUpdate {
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

    /** Sets the opt-in retention period of a category in years; `null` removes it. */
    fun setRetentionYears(categoryId: CategoryIdModel, years: Int?): CatalogUpdate {
        val category = categories[categoryId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND)
        return applied(categories = categories + (categoryId to category.copy(retentionYears = years)))
    }

    /** Removes an unused category without children. */
    fun removeCategory(categoryId: CategoryIdModel): CatalogUpdate {
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
    fun renameTag(from: TagNameModel, to: TagNameModel): CatalogUpdate {
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
    fun removeTag(tag: TagNameModel): CatalogUpdate {
        if (tag !in tags) {
            return CatalogUpdate.Rejected(reason = CatalogRejection.TAG_NOT_FOUND)
        }
        val untagged = documents.mapValues { (_, document) -> document.copy(tags = document.tags - tag) }
        return applied(documents = untagged, tags = tags - tag)
    }

    // --- Helpers

    private fun updateDocument(
        documentId: DocumentIdModel,
        addedTags: Set<TagNameModel> = emptySet(),
        change: (DocumentModel) -> DocumentModel,
    ): CatalogUpdate {
        val document = documents[documentId]
            ?: return CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND)
        return applied(documents = documents + (documentId to change(document)), tags = tags + addedTags)
    }

    private fun applied(
        documents: Map<DocumentIdModel, DocumentModel> = this.documents,
        categories: Map<CategoryIdModel, CategoryModel> = this.categories,
        tags: Set<TagNameModel> = this.tags,
        links: Set<DocumentLinkModel> = this.links,
        settings: CatalogSettingsModel = this.settings,
    ): CatalogUpdate {
        val catalog = copy(
            revision = revision.next(),
            documents = documents,
            categories = categories,
            tags = tags,
            links = links,
            settings = settings,
        )
        return CatalogUpdate.Applied(catalog = catalog)
    }

    /** Uses the spelling the catalog already knows for tags that differ only in casing. */
    private fun canonicalTags(requested: Set<TagNameModel>): Set<TagNameModel> {
        return requested.map { tag -> tags.firstOrNull { it == tag } ?: tag }.toSet()
    }

    private fun isNameTaken(name: String, parentId: CategoryIdModel?, except: CategoryIdModel?): Boolean {
        val key = name.trim().lowercase(Locale.ROOT)
        return categories.values.any {
            it.id != except && it.parentId == parentId && it.name.trim().lowercase(Locale.ROOT) == key
        }
    }

    companion object {

        fun empty(libraryId: LibraryIdModel): CatalogModel {
            return CatalogModel(
                libraryId = libraryId,
                revision = CatalogRevisionModel.INITIAL,
                documents = emptyMap(),
                categories = emptyMap(),
                tags = emptySet(),
            )
        }

        /** `true` if following the parents of [categoryId] leads back to it. */
        private fun isInCycle(categoryId: CategoryIdModel, parents: Map<CategoryIdModel, CategoryModel>): Boolean {
            val visited = mutableSetOf<CategoryIdModel>()
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
