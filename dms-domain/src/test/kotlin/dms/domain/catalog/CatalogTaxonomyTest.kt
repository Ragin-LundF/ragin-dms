package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class CatalogTaxonomyTest {

    private val finance = Category(id = CategoryId(value = "finance"), name = "Finance")
    private val invoices = Category(id = CategoryId(value = "invoices"), name = "Invoices", parentId = finance.id)
    private val taxonomy = Catalog.empty(libraryId = TestData.LIBRARY_ID)
        .addCategory(category = finance).applied()
        .addCategory(category = invoices).applied()

    @Test
    fun `a category needs a known parent and a unique name among its siblings`() {
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND),
            actual = taxonomy.addCategory(
                category = Category(id = CategoryId(value = "x"), name = "X", parentId = CategoryId(value = "nope")),
            ),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NAME_TAKEN),
            actual = taxonomy.addCategory(
                category = Category(id = CategoryId(value = "x"), name = " invoices ", parentId = finance.id),
            ),
        )
    }

    @Test
    fun `the same name is allowed below different parents`() {
        val other = Category(id = CategoryId(value = "other"), name = "Invoices")

        val catalog = taxonomy.addCategory(category = other).applied()

        assertEquals(expected = 3, actual = catalog.categories.size)
    }

    @Test
    fun `moving a category below its own descendant is rejected`() {
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_CYCLE),
            actual = taxonomy.moveCategory(categoryId = finance.id, parentId = invoices.id),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_CYCLE),
            actual = taxonomy.moveCategory(categoryId = finance.id, parentId = finance.id),
        )
    }

    @Test
    fun `a category in use or with children cannot be removed`() {
        val document = TestData.document(id = "d1", path = "a.pdf", categoryId = invoices.id)
        val catalog = taxonomy.addDocument(document = document).applied()

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_IN_USE),
            actual = catalog.removeCategory(categoryId = invoices.id),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_HAS_CHILDREN),
            actual = catalog.removeCategory(categoryId = finance.id),
        )
    }

    @Test
    fun `assigning and clearing a category`() {
        val document = TestData.document(id = "d1", path = "a.pdf")
        val assigned = taxonomy.addDocument(document = document).applied()
            .assignCategory(documentId = document.id, categoryId = invoices.id).applied()

        assertEquals(expected = invoices.id, actual = assigned.documents.getValue(key = document.id).categoryId)
        val cleared = assigned.assignCategory(documentId = document.id, categoryId = null).applied()
        assertEquals(expected = null, actual = cleared.documents.getValue(key = document.id).categoryId)
    }

    @Test
    fun `renaming a tag into an existing tag merges both`() {
        val tax = TagName.of(raw = "Tax")
        val taxes = TagName.of(raw = "Taxes")
        val first = TestData.document(id = "d1", path = "a.pdf", tags = setOf(tax))
        val second = TestData.document(id = "d2", path = "b.pdf", tags = setOf(taxes))

        val merged = TestData.catalogWith(first, second).renameTag(from = taxes, to = TagName.of(raw = "tax")).applied()

        assertEquals(expected = listOf("Tax"), actual = merged.tags.map { it.value })
        assertTrue(actual = merged.documents.values.all { it.tags.single().value == "Tax" })
    }

    @Test
    fun `renaming a tag to another casing changes its spelling`() {
        val document = TestData.document(id = "d1", path = "a.pdf", tags = setOf(TagName.of(raw = "tax")))

        val renamed = TestData.catalogWith(document)
            .renameTag(from = TagName.of(raw = "tax"), to = TagName.of(raw = "Tax")).applied()

        assertEquals(expected = "Tax", actual = renamed.tags.single().value)
        assertEquals(expected = "Tax", actual = renamed.documents.getValue(key = document.id).tags.single().value)
    }

    @Test
    fun `removing a tag removes it from every document`() {
        val tag = TagName.of(raw = "Draft")
        val document = TestData.document(id = "d1", path = "a.pdf", tags = setOf(tag))

        val catalog = TestData.catalogWith(document).removeTag(tag = tag).applied()

        assertTrue(actual = catalog.tags.isEmpty())
        assertTrue(actual = catalog.documents.getValue(key = document.id).tags.isEmpty())
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.TAG_NOT_FOUND),
            actual = catalog.removeTag(tag = tag),
        )
    }
}
