package dms.ui.review

import dms.application.classification.model.ApprovedCategory
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.review.model.ReviewDraftModel
import dms.ui.review.model.ReviewFieldModel
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ReviewDraftsTest {

    private val invoices = CategoryModel(id = CategoryIdModel(value = "invoices"), name = "Invoices")
    private val energy = CategoryModel(id = CategoryIdModel(value = "energy"), name = "Energy", parentId = invoices.id)
    private val catalog = CatalogModel(
        libraryId = LibraryIdModel(value = "sample-library"),
        revision = CatalogRevisionModel(value = 1),
        documents = emptyMap(),
        categories = listOf(invoices, energy).associateBy { it.id },
        tags = emptySet(),
    )
    private val current = draft(fileName = "scan-7.pdf", accepted = false)

    @Test
    fun `only checked fields are approved, and an unchanged file name is left out`() {
        val draft = draft(fileName = "scan-7.pdf").copy(title = ReviewFieldModel(accepted = false, value = "Ignored"))

        val approval = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog).getOrThrow()

        assertNull(actual = approval.title)
        assertNull(actual = approval.fileName)
        assertEquals(expected = LocalDate.parse("2024-03-15"), actual = approval.documentDate)
        assertEquals(
            expected = setOf(TagNameModel.of(raw = "Tax"), TagNameModel.of(raw = "Home")),
            actual = approval.tags,
        )
        assertEquals(expected = ApprovedCategory.Existing(categoryId = energy.id), actual = approval.category)
    }

    @Test
    fun `a typed path that does not exist is approved with every level`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(category = ReviewFieldModel(accepted = true, value = "Insurance / Household/ Rent and living"))

        val approval = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog).getOrThrow()

        val expected = ApprovedCategory.Path(names = listOf("Insurance", "Household", "Rent and living"))
        assertEquals(expected = expected, actual = approval.category)
    }

    @Test
    fun `a typed path with an empty level is refused with a hint`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(category = ReviewFieldModel(accepted = true, value = "Insurance // Rent"))

        val result = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog)

        assertTrue(actual = result.exceptionOrNull()?.message.orEmpty().contains(other = "needs a name"))
    }

    @Test
    fun `a date in another format is refused with a hint`() {
        val draft = draft(fileName = "scan-7.pdf")
            .copy(documentDate = ReviewFieldModel(accepted = true, value = "15.03.2024"))

        val result = ReviewDrafts.approvalOf(draft = draft, current = current, catalog = catalog)

        assertTrue(actual = result.exceptionOrNull()?.message.orEmpty().contains(other = "YYYY-MM-DD"))
    }

    @Test
    fun `the suggested file name starts checked only when the settings ask for it`() {
        val suggestion = ClassificationSuggestionModel(
            documentId = DocumentIdModel(value = "a"),
            model = "sample-model",
            fileName = "2024-03-15 Sample bill.pdf",
        )

        val optional = ReviewDrafts.draftOf(suggestion = suggestion, catalog = catalog, acceptFileName = false)
        val taken = ReviewDrafts.draftOf(suggestion = suggestion, catalog = catalog, acceptFileName = true)

        assertEquals(
            expected = ReviewFieldModel(accepted = false, value = "2024-03-15 Sample bill.pdf"),
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

    private fun draft(fileName: String, accepted: Boolean = true): ReviewDraftModel {
        return ReviewDraftModel(
            title = ReviewFieldModel(accepted = accepted, value = "Sample bill"),
            documentDate = ReviewFieldModel(accepted = accepted, value = "2024-03-15"),
            correspondent = ReviewFieldModel(accepted = accepted, value = "Sample Energy"),
            category = ReviewFieldModel(accepted = accepted, value = "invoices / energy"),
            tags = ReviewFieldModel(accepted = accepted, value = "Tax, Home"),
            fileName = ReviewFieldModel(accepted = accepted, value = fileName),
        )
    }
}
