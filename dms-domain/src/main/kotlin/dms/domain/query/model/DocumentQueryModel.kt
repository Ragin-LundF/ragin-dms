package dms.domain.query.model

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import java.time.Instant
import java.time.LocalDate
import java.util.Locale

/**
 * Which documents to show, export, restructure or ask about. Every condition that is set must hold;
 * an empty query matches every document.
 *
 * [text] looks at the path, title, correspondent, tags and note. [category] includes its
 * subcategories. [dates] applies to the document date, or to the day the document was added when it
 * has none. [contentText] is a term in the documents' text; [matches] ignores it, because only
 * the content search can read the texts.
 */
data class DocumentQueryModel(
    val text: String? = null,
    val category: CategoryIdModel? = null,
    val uncategorized: Boolean = false,
    val tags: TagFilterModel = TagFilterModel.NONE,
    val untagged: Boolean = false,
    val dates: DateRangeModel? = null,
    val correspondent: String? = null,
    val folder: RelativePathModel? = null,
    val fields: List<FieldConditionModel> = emptyList(),
    val addedSince: Instant? = null,
    val contentText: String? = null,
) {
    init {
        require(value = !(uncategorized && category != null)) { "A query cannot ask for a category and none" }
    }

    /** `true` if the query needs the documents' text through the content search. */
    val needsContent: Boolean
        get() {
            return !contentText.isNullOrBlank()
        }

    fun matches(document: DocumentModel, context: QueryContextModel): Boolean {
        return matchesText(document = document) &&
            matchesCategory(document = document, catalog = context.catalog) &&
            matchesTags(document = document) &&
            (dates == null || dates.contains(date = dateOf(document = document, context = context))) &&
            matchesCorrespondent(document = document) &&
            (folder == null || document.path.isInside(folder = folder)) &&
            fields.all { it.matches(fields = document.fields, today = context.today) } &&
            (addedSince == null || document.addedAt >= addedSince)
    }

    /**
     * `true` if the query names a category, tag or field the catalog no longer has, for example after
     * the category was removed; such a smart folder is shown as invalid instead of silently empty.
     */
    fun refersToMissing(catalog: CatalogModel): Boolean {
        val tagsUsed = tags.all + tags.any + tags.none
        return (category != null && category !in catalog.categories) ||
            !catalog.tags.containsAll(elements = tagsUsed) ||
            fields.any { catalog.settings.field(key = it.key) == null }
    }

    /** The documents of [context]'s catalog this query matches, by path. */
    fun filter(context: QueryContextModel): List<DocumentModel> {
        return context.catalog.documents.values
            .filter { matches(document = it, context = context) }
            .sortedBy { it.path.value }
    }

    private fun matchesText(document: DocumentModel): Boolean {
        val wanted = text?.trim()?.lowercase(Locale.ROOT)
        if (wanted.isNullOrEmpty()) {
            return true
        }
        val haystack = listOfNotNull(
            document.path.value,
            document.metadata.title,
            document.metadata.correspondent,
            document.note,
        ) + document.tags.map { it.value }
        return haystack.any { it.lowercase(Locale.ROOT).contains(other = wanted) }
    }

    private fun matchesCategory(document: DocumentModel, catalog: CatalogModel): Boolean {
        if (uncategorized) {
            return document.categoryId == null
        }
        val wanted = category ?: return true
        var current = document.categoryId
        val visited = mutableSetOf<CategoryIdModel>()
        while (current != null && visited.add(element = current)) {
            if (current == wanted) {
                return true
            }
            current = catalog.categories[current]?.parentId
        }
        return false
    }

    private fun matchesTags(document: DocumentModel): Boolean {
        if (untagged && document.tags.isNotEmpty()) {
            return false
        }
        return tags.matches(tags = document.tags)
    }

    private fun matchesCorrespondent(document: DocumentModel): Boolean {
        val wanted = correspondent?.trim()?.lowercase(Locale.ROOT)
        if (wanted.isNullOrEmpty()) {
            return true
        }
        return document.metadata.correspondent?.trim()?.lowercase(Locale.ROOT) == wanted
    }

    private fun dateOf(document: DocumentModel, context: QueryContextModel): LocalDate {
        return document.metadata.documentDate ?: document.addedAt.atZone(context.zone).toLocalDate()
    }

    companion object {
        val ALL: DocumentQueryModel = DocumentQueryModel()
    }
}
