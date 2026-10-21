package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.document.DocumentMetadata
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertSame
import kotlin.test.assertTrue

class CatalogMergerTest {

    private val shared = TestData.document(id = "shared", path = "shared.pdf")
    private val base = TestData.catalogWith(shared)

    @Test
    fun `identical copies merge without a new revision`() {
        val result = CatalogMerger.merge(first = base, second = base.copy())

        assertSame(expected = base, actual = result.catalog)
        assertTrue(actual = result.conflicts.isEmpty())
    }

    @Test
    fun `entries added on either side are kept and nothing is deleted`() {
        val left = base.addDocument(document = TestData.document(id = "left", path = "left.pdf")).applied()
        val right = base.addDocument(document = TestData.document(id = "right", path = "right.pdf")).applied()

        val result = CatalogMerger.merge(first = left, second = right)

        assertEquals(
            expected = setOf("shared", "left", "right"),
            actual = result.catalog.documents.keys.map { it.value }.toSet(),
        )
        assertEquals(expected = left.revision.next(), actual = result.catalog.revision)
        assertTrue(actual = result.conflicts.isEmpty())
    }

    @Test
    fun `the copy with the higher revision wins a differing document`() {
        val older = base.updateMetadata(documentId = shared.id, metadata = DocumentMetadata(title = "Old")).applied()
        val newer = older
            .updateMetadata(documentId = shared.id, metadata = DocumentMetadata(title = "New")).applied()

        val result = CatalogMerger.merge(first = older, second = newer)

        assertEquals(expected = "New", actual = result.catalog.documents.getValue(key = shared.id).metadata.title)
        val conflict = result.conflicts.single() as CatalogMergeConflict.DocumentDiffers
        assertEquals(expected = "Old", actual = conflict.discarded.metadata.title)
    }

    @Test
    fun `two documents on the same path are reported as a path clash`() {
        val left = base.addDocument(document = TestData.document(id = "left", path = "same.pdf")).applied()
        val right = left.removeDocument(documentId = TestData.document(id = "left", path = "same.pdf").id).applied()
            .addDocument(document = TestData.document(id = "right", path = "same.pdf")).applied()

        val result = CatalogMerger.merge(first = left, second = right)

        val clash = result.conflicts.single() as CatalogMergeConflict.PathClash
        assertEquals(expected = "right", actual = clash.kept.id.value)
        assertEquals(expected = "left", actual = clash.discarded.id.value)
    }

    @Test
    fun `categories are merged with the newer copy winning`() {
        val category = Category(id = CategoryId(value = "c"), name = "Letters")
        val older = base.addCategory(category = category).applied()
        val newer = older.renameCategory(categoryId = category.id, name = "Mail").applied()
            .addCategory(category = Category(id = CategoryId(value = "n"), name = "New")).applied()

        val result = CatalogMerger.merge(first = newer, second = older)

        assertEquals(expected = "Mail", actual = result.catalog.categories.getValue(key = category.id).name)
        assertEquals(expected = 2, actual = result.catalog.categories.size)
        assertTrue(actual = result.conflicts.single() is CatalogMergeConflict.CategoryDiffers)
    }

    @Test
    fun `copies of different libraries cannot be merged`() {
        val other = Catalog.empty(libraryId = LibraryId(value = "other"))

        assertFailsWith<IllegalArgumentException> { CatalogMerger.merge(first = base, second = other) }
    }
}
