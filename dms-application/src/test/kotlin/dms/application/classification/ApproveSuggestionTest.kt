package dms.application.classification

import dms.application.TestLibrary
import dms.application.classification.model.ApprovedCategory
import dms.application.classification.model.SuggestionApproval
import dms.application.common.OperationOutcome
import dms.application.fake.ai.FakeClassifier
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.ClassificationSource
import dms.domain.document.Document
import dms.domain.taxonomy.TagName
import kotlinx.coroutines.test.runTest
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ApproveSuggestionTest {

    private val approval = SuggestionApproval(
        title = "Synthetic statement",
        documentDate = LocalDate.of(2024, 2, 15),
        category = ApprovedCategory.Path(names = listOf("Statements")),
        tags = setOf(TagName.of(raw = "Tax")),
        fileName = "2024-02 statement.pdf",
    )

    @Test
    fun `approved fields are written, the file is renamed everywhere and the suggestion is done`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/scan-001.pdf", content = "scan")
            suggestFor(library = library, document = document)

            val outcome = library.approveSuggestion(documentId = document.id, approval = approval)

            val catalog = assertIs<OperationOutcome.Done>(value = outcome).catalog
            val approved = catalog.documents.getValue(key = document.id)
            assertEquals(expected = "inbox/2024-02 statement.pdf", actual = approved.path.value)
            assertEquals(expected = "Synthetic statement", actual = approved.metadata.title)
            assertEquals(expected = LocalDate.of(2024, 2, 15), actual = approved.metadata.documentDate)
            assertEquals(expected = "Statements", actual = catalog.categories[approved.categoryId]?.name)
            assertEquals(expected = setOf(TagName.of(raw = "tax")), actual = approved.tags)
            assertEquals(expected = ClassificationSource.AI, actual = approved.classification?.source)
            assertEquals(expected = FakeClassifier.MODEL, actual = approved.classification?.model)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = setOf("inbox/2024-02 statement.pdf"),
                    actual = library.folders.paths(folder = folder),
                )
            }
            assertTrue(actual = library.tasks.suggestions().isEmpty())
        }
    }

    @Test
    fun `a taken file name refuses the approval and changes nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "inbox/2024-02 statement.pdf", content = "other")
            val document = library.add(path = "inbox/scan-001.pdf", content = "scan")
            suggestFor(library = library, document = document)
            val before = library.catalog

            val outcome = library.approveSuggestion(documentId = document.id, approval = approval)

            assertEquals(expected = OperationOutcome.NameTaken, actual = outcome)
            assertEquals(expected = before, actual = library.catalog)
            assertEquals(expected = 1, actual = library.tasks.suggestions().size)
        }
    }

    @Test
    fun `a new category with the name of an existing one uses the existing one`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val first = library.add(path = "inbox/a.pdf", content = "a")
            val second = library.add(path = "inbox/b.pdf", content = "b")
            val categoryOnly = SuggestionApproval(
                category = ApprovedCategory.Path(names = listOf("Statements")),
            )
            suggestFor(library = library, document = first)
            suggestFor(library = library, document = second)

            library.approveSuggestion(documentId = first.id, approval = categoryOnly)
            val outcome = library.approveSuggestion(
                documentId = second.id,
                approval = categoryOnly.copy(category = ApprovedCategory.Path(names = listOf("statements"))),
            )

            val catalog = assertIs<OperationOutcome.Done>(value = outcome).catalog
            assertEquals(expected = 1, actual = catalog.categories.size)
            assertEquals(
                expected = catalog.documents.getValue(key = first.id).categoryId,
                actual = catalog.documents.getValue(key = second.id).categoryId,
            )
        }
    }

    @Test
    fun `a path creates every missing level and assigns the deepest one`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/a.pdf", content = "a")
            suggestFor(library = library, document = document)
            val path = ApprovedCategory.Path(names = listOf("Insurance", "Household", "Rent and living"))

            val outcome = library.approveSuggestion(
                documentId = document.id,
                approval = SuggestionApproval(category = path),
            )

            val catalog = assertIs<OperationOutcome.Done>(value = outcome).catalog
            assertEquals(expected = 3, actual = catalog.categories.size)
            val leafId = checkNotNull(value = catalog.documents.getValue(key = document.id).categoryId)
            val leaf = catalog.categories.getValue(key = leafId)
            val middle = catalog.categories.getValue(key = checkNotNull(value = leaf.parentId))
            val top = catalog.categories.getValue(key = checkNotNull(value = middle.parentId))
            assertEquals(
                expected = listOf("Insurance", "Household", "Rent and living"),
                actual = listOf(top.name, middle.name, leaf.name),
            )
            assertNull(actual = top.parentId)
        }
    }

    @Test
    fun `a path reuses the levels that exist, ignoring case, and creates only the rest`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val first = library.add(path = "inbox/a.pdf", content = "a")
            val second = library.add(path = "inbox/b.pdf", content = "b")
            suggestFor(library = library, document = first)
            suggestFor(library = library, document = second)
            library.approveSuggestion(
                documentId = first.id,
                approval = SuggestionApproval(category = ApprovedCategory.Path(names = listOf("Insurance", "Car"))),
            )
            val catalog = assertIs<OperationOutcome.Done>(
                value = library.approveSuggestion(
                    documentId = second.id,
                    approval = SuggestionApproval(
                        category = ApprovedCategory.Path(names = listOf("insurance", "Household", "Rent")),
                    ),
                ),
            ).catalog

            val insurance = catalog.categories.values.single { it.name == "Insurance" }
            val household = catalog.categories.values.single { it.name == "Household" }
            val rent = catalog.categories.values.single { it.name == "Rent" }
            assertEquals(expected = 4, actual = catalog.categories.size)
            assertEquals(expected = insurance.id, actual = household.parentId)
            assertEquals(expected = household.id, actual = rent.parentId)
            assertEquals(expected = rent.id, actual = catalog.documents.getValue(key = second.id).categoryId)
        }
    }

    private suspend fun suggestFor(library: TestLibrary, document: Document) {
        library.tasks.saveSuggestion(
            suggestion = ClassificationSuggestion(documentId = document.id, model = FakeClassifier.MODEL),
        )
    }
}
