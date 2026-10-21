package dms.ui.library

import dms.domain.catalog.Catalog
import dms.domain.document.Document
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewState
import java.time.Duration
import java.time.Instant
import java.time.ZoneId
import java.util.Locale

/** Which documents the library shows: the sidebar's selection, the year and the text filter together. */
object LibraryFilter {

    /** How far back "Recently added" reaches. */
    private val RECENT: Duration = Duration.ofDays(30)

    /** The documents [view] selects; recent ones newest first, all others by path. */
    fun documents(catalog: Catalog, view: LibraryViewState, now: Instant, zone: ZoneId): List<Document> {
        val query = view.query.trim().lowercase(Locale.ROOT)
        val year = view.year
        val shown = catalog.documents.values.filter { document ->
            matches(document = document, selection = view.selection, now = now) &&
                (year == null || year.matches(year = yearOf(document = document, zone = zone))) &&
                matches(document = document, query = query)
        }
        return if (view.selection == LibrarySelection.Recent) {
            shown.sortedByDescending { it.addedAt }
        } else {
            shown.sortedBy { it.path.value.lowercase(Locale.ROOT) }
        }
    }

    /** The year a document belongs to: of its document date, or of when it was added. */
    fun yearOf(document: Document, zone: ZoneId): Int {
        return document.metadata.documentDate?.year ?: document.addedAt.atZone(zone).year
    }

    /** Every year a document belongs to, newest first. */
    fun years(documents: Collection<Document>, zone: ZoneId): List<Int> {
        return documents.map { yearOf(document = it, zone = zone) }.distinct().sortedDescending()
    }

    fun titleOf(selection: LibrarySelection, catalog: Catalog): String {
        return when (selection) {
            is LibrarySelection.Folder -> if (selection.path.isRoot) "All documents" else selection.path.fileName
            is LibrarySelection.Category -> catalog.categories[selection.categoryId]?.name ?: "Category"
            LibrarySelection.Uncategorized -> "Uncategorized"
            is LibrarySelection.Tag -> "#${selection.tag.value}"
            LibrarySelection.Recent -> "Recently added"
        }
    }

    private fun matches(document: Document, selection: LibrarySelection, now: Instant): Boolean {
        return when (selection) {
            is LibrarySelection.Folder -> document.path.isInside(folder = selection.path)
            is LibrarySelection.Category -> document.categoryId == selection.categoryId
            LibrarySelection.Uncategorized -> document.categoryId == null
            is LibrarySelection.Tag -> selection.tag in document.tags
            LibrarySelection.Recent -> document.addedAt >= now.minus(RECENT)
        }
    }

    private fun matches(document: Document, query: String): Boolean {
        if (query.isEmpty()) {
            return true
        }
        val fields = listOfNotNull(
            document.path.value,
            document.metadata.title,
            document.metadata.correspondent,
        ) + document.tags.map { it.value }
        return fields.any { it.lowercase(Locale.ROOT).contains(other = query) }
    }
}
