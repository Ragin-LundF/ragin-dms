package dms.ui.shell.palette

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.model.LibrarySelection
import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.palette.model.PaletteItemModel
import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PaletteItemsTest {

    private val energy = TagNameModel.of(raw = "Energy")
    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
    private val power = document(id = "a", path = "Home/Energy/power.pdf", title = "Power bill", tags = setOf(energy))
    private val water = document(id = "b", path = "Home/water.pdf", title = null, tags = emptySet())
    private val catalog = CatalogModel(
        libraryId = LibraryIdModel(value = "library"),
        revision = CatalogRevisionModel.INITIAL,
        documents = mapOf(power.id to power, water.id to water),
        categories = mapOf(bills.id to bills),
        tags = setOf(energy),
    )

    @Test
    fun `documents are found by title or file name, and by text after them`() {
        val groups = PaletteItems.of(query = "POWER", catalog = catalog, textMatches = setOf(power.id, water.id))

        val documents = groups.single { it.title == "Documents" }.items.map { it as PaletteItemModel.OpenDocument }
        assertEquals(expected = listOf(power.id, water.id), actual = documents.map { it.documentId })
        assertEquals(expected = "Power bill", actual = documents.first().label)
        assertTrue(actual = documents.last().detail.startsWith(prefix = "Found in its text"))
    }

    @Test
    fun `places carry their kind and count`() {
        val places = PaletteItems.of(query = "energy", catalog = catalog, textMatches = emptySet())
            .single { it.title == "Go to" }.items.map { it as PaletteItemModel.GoTo }

        assertEquals(
            expected = listOf(
                PaletteItemModel.GoTo(
                    selection = LibrarySelection.Tag(tag = energy),
                    label = "Energy",
                    kind = "Tag",
                    count = 1,
                ),
                PaletteItemModel.GoTo(
                    selection = LibrarySelection.Folder(path = RelativePathModel.of(raw = "Home/Energy")),
                    label = "Home/Energy",
                    kind = "Folder",
                    count = 1,
                ),
            ),
            actual = places,
        )
    }

    @Test
    fun `an empty query lists every action and no documents`() {
        val groups = PaletteItems.of(query = " ", catalog = catalog, textMatches = emptySet())

        assertEquals(expected = listOf("Actions"), actual = groups.map { it.title })
        assertEquals(expected = PaletteAction.entries.size, actual = groups.single().items.size)
    }

    @Test
    fun `a query always ends with asking the library`() {
        val actions = PaletteItems.of(query = "scan", catalog = null, textMatches = emptySet()).single().items

        assertEquals(
            expected = listOf(
                PaletteItemModel.Run(action = PaletteAction.SCAN),
                PaletteItemModel.AskLibrary(question = "scan"),
            ),
            actual = actions,
        )
    }

    private fun document(id: String, path: String, title: String?, tags: Set<TagNameModel>): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = RelativePathModel.of(raw = path),
            contentHash = ContentHashModel(value = id.padStart(length = 64, padChar = '0')),
            size = 1,
            addedAt = Instant.parse("2026-01-01T10:00:00Z"),
            metadata = DocumentMetadataModel(title = title),
            tags = tags,
        )
    }
}
