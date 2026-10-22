package dms.ui.review

import dms.application.classification.model.ApprovedCategory
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRevision
import dms.domain.catalog.LibraryId
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.DocumentId
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.review.model.ReviewDraft
import dms.ui.review.model.ReviewField
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
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
        assertEquals(expected = ApprovedCategory.Existing(categoryId = energy.id), actual = approval.category)
    }

    @Test
    fun `a typed path that does not exist is approved with every level`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(category = ReviewField(accepted = true, value = "Insurance / Household/ Rent and living"))

        val approval = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog).getOrThrow()

        val expected = ApprovedCategory.Path(names = listOf("Insurance", "Household", "Rent and living"))
        assertEquals(expected = expected, actual = approval.category)
    }

    @Test
    fun `a typed path with an empty level is refused with a hint`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(category = ReviewField(accepted = true, value = "Insurance // Rent"))

        val result = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog)

        assertTrue(actual = result.exceptionOrNull()?.message.orEmpty().contains(other = "needs a name"))
    }

    @Test
    fun `a date in another format is refused with a hint`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(documentDate = ReviewField(accepted = true, value = "15.03.2024"))

        val result = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog)

        assertTrue(actual = result.exceptionOrNull()?.message.orEmpty().contains(other = "YYYY-MM-DD"))
    }

    @Test
    fun `the suggested file name starts checked only when the settings ask for it`() {
        val suggestion = ClassificationSuggestion(
            documentId = DocumentId(value = "a"),
            model = "sample-model",
            fileName = "2024-03-15 Sample bill.pdf",
        )

        val optional = ReviewDrafts.draftOf(suggestion = suggestion, catalog = catalog, acceptFileName = false)
        val taken = ReviewDrafts.draftOf(suggestion = suggestion, catalog = catalog, acceptFileName = true)

        assertEquals(
            expected = ReviewField(accepted = false, value = "2024-03-15 Sample bill.pdf"),
            actual = optional.fileName,
        )
        assertTrue(actual = taken.fileName.accepted)
    }

    @Test
    fun `categories are offered as sorted paths, and only an unknown one is new`() {
        val paths = ReviewDrafts.categoryPathsOf(catalog = catalog)

        assertEquals(expected = listOf("Invoices", "Invoices / Energy"), actual = paths)
        assertFalse(actual = ReviewDrafts.isNewCategory(text = "invoices / energy", paths = paths))
        assertFalse(actual = ReviewDrafts.isNewCategory(text = "Energy", paths = paths))
        assertFalse(actual = ReviewDrafts.isNewCategory(text = " ", paths = paths))
        assertTrue(actual = ReviewDrafts.isNewCategory(text = "Invoices / Water", paths = paths))
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
