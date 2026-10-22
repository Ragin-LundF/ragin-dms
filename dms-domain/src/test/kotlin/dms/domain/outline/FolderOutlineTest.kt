package dms.domain.outline

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.RelativePathModel
import dms.domain.outline.model.FolderOutlineModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class FolderOutlineTest {

    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
    private val inbox = RelativePathModel.of(raw = "Inbox")

    private fun catalog(vararg paths: String): CatalogModel {
        val base = TestData.catalogWith().addCategory(category = bills).applied()
        return paths.foldIndexed(initial = base) { index, catalog, path ->
            val categoryId = if (path.startsWith(prefix = "Home/")) bills.id else null
            catalog.addDocument(
                document = TestData.document(id = "d$index", path = path, categoryId = categoryId),
            ).applied()
        }
    }

    @Test
    fun `folders carry counts, the most common category and sample titles, without the inbox`() {
        val outline = FolderOutlineModel.of(
            catalog = catalog("Home/Power/a.pdf", "Home/Water/b.pdf", "Inbox/new.pdf", "Car/c.pdf"),
            excluded = listOf(inbox),
        )

        assertEquals(
            expected = listOf("Car", "Home", "Home/Power", "Home/Water"),
            actual = outline.folders.map { it.path.value },
        )
        val home = outline.folders.first { it.path.value == "Home" }
        assertEquals(expected = 2, actual = home.documentCount)
        assertEquals(expected = bills.id, actual = home.topCategory)
        assertEquals(expected = setOf("a.pdf", "b.pdf"), actual = home.sampleTitles.toSet())
    }

    @Test
    fun `a large tree is cut at a depth and the cut folders count what they hide`() {
        val paths = (1..30).flatMap { top -> (1..10).map { sub -> "T$top/S$sub/f.pdf" } }.toTypedArray()

        val outline = FolderOutlineModel.of(catalog = catalog(*paths), excluded = emptyList(), limit = 200)

        assertEquals(expected = 30, actual = outline.folders.size)
        assertEquals(expected = 10, actual = outline.folders.first().hiddenSubfolders)
    }

    @Test
    fun `new folders are allowed directly below a known folder, never in the inbox`() {
        val outline = FolderOutlineModel.of(catalog = catalog("Home/a.pdf"), excluded = listOf(inbox))

        assertTrue(actual = outline.canCreate(path = RelativePathModel.of(raw = "Home/Garden")))
        assertTrue(actual = outline.canCreate(path = RelativePathModel.of(raw = "Garden")))
        assertFalse(actual = outline.canCreate(path = RelativePathModel.of(raw = "Away/Garden")))
        assertFalse(actual = outline.canCreate(path = RelativePathModel.of(raw = "Inbox/Sorted")))
        assertFalse(actual = outline.canCreate(path = inbox))
    }
}
