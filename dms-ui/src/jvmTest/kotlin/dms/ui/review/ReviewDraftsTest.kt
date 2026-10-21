package dms.ui.review

import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRevision
import dms.domain.catalog.LibraryId
import dms.domain.classification.SuggestedCategory
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.review.model.ReviewDraft
import dms.ui.review.model.ReviewField
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ReviewDraftsTest {

    private val invoices = Category(id = CategoryId(value = "invoices"), name = "Invoices")
    private val energy = Category(id = CategoryId(value = "energy"), name = "Energy", parentId = invoices.id)
    private val catalog = Catalog(
        libraryId = LibraryId(value = "sample-library"),
        revision = CatalogRevision(value = 1),
        documents = emptyMap(),
        categories = listOf(invoices, energy).associateBy { it.id },
        tags = emptySet(),
    )
    private val current = draft(fileName = "scan-7.pdf", accepted = false)

    @Test
    fun `only checked fields are approved, and an unchanged file name is left out`() {
        val draft = draft(fileName = "scan-7.pdf").copy(title = ReviewField(accepted = false, value = "Ignored"))

        val approval = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog).getOrThrow()

        assertNull(actual = approval.title)
        assertNull(actual = approval.fileName)
        assertEquals(expected = LocalDate.parse("2024-03-15"), actual = approval.documentDate)
        assertEquals(expected = setOf(TagName.of(raw = "Tax"), TagName.of(raw = "Home")), actual = approval.tags)
        assertEquals(expected = SuggestedCategory.Existing(categoryId = energy.id), actual = approval.category)
    }

    @Test
    fun `a category below an existing parent is created there`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(category = ReviewField(accepted = true, value = "Invoices / Water"))

        val approval = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog).getOrThrow()

        val expected = SuggestedCategory.New(name = "Water", parentId = invoices.id)
        assertEquals(expected = expected, actual = approval.category)
    }

    @Test
    fun `a date in another format is refused with a hint`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(documentDate = ReviewField(accepted = true, value = "15.03.2024"))

        val result = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog)

        assertTrue(actual = result.exceptionOrNull()?.message.orEmpty().contains(other = "YYYY-MM-DD"))
    }

    private fun draft(fileName: String, accepted: Boolean = true): ReviewDraft {
        return ReviewDraft(
            title = ReviewField(accepted = accepted, value = "Sample bill"),
            documentDate = ReviewField(accepted = accepted, value = "2024-03-15"),
            correspondent = ReviewField(accepted = accepted, value = "Sample Energy"),
            category = ReviewField(accepted = accepted, value = "invoices / energy"),
            tags = ReviewField(accepted = accepted, value = "Tax, Home"),
            fileName = ReviewField(accepted = accepted, value = fileName),
        )
    }
}
