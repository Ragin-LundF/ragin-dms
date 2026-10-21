package dms.application.classification

import dms.application.TestLibrary
import dms.application.classification.model.SuggestOutcome
import dms.application.fake.ai.FakeClassifier
import dms.application.port.ai.model.ClassificationResult
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.SuggestedTag
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SuggestClassificationTest {

    @Test
    fun `a suggestion waits for review and the text is stored in every folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/scan-001.pdf", content = "scan")
            library.classifier.result = ClassificationResult.Suggested(
                suggestion = ClassificationSuggestion(
                    documentId = document.id,
                    model = FakeClassifier.MODEL,
                    title = "Synthetic statement",
                    category = SuggestedCategory.Existing(categoryId = CategoryId(value = "unknown")),
                    tags = listOf(SuggestedTag(name = TagName.of(raw = "Tax"), isNew = false)),
                ),
            )

            val outcome = library.suggestClassification(documentId = document.id)

            val suggestion = assertIs<SuggestOutcome.Suggested>(value = outcome).suggestion
            assertNull(actual = suggestion.category)
            assertEquals(expected = listOf(true), actual = suggestion.tags.map { it.isNew })
            assertEquals(expected = listOf(suggestion), actual = library.tasks.suggestions())
            assertTrue(actual = library.catalog.documents.getValue(key = document.id).classification == null)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "synthetic text",
                    actual = library.folders.textIn(folder = folder, documentId = document.id),
                )
            }
        }
    }

    @Test
    fun `the request carries the library's file names as examples`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/2024-01 first.pdf", content = "a")
            val document = library.add(path = "inbox/scan-002.pdf", content = "b")

            library.suggestClassification(documentId = document.id)

            val request = library.classifier.requests.single()
            assertEquals(expected = listOf("2024-01 first.pdf"), actual = request.fileNameExamples)
            assertEquals(expected = "scan-002.pdf", actual = request.currentFileName)
        }
    }

    @Test
    fun `a file type the AI cannot read is never sent`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "notes/plain.txt", content = "text")

            val outcome = library.suggestClassification(documentId = document.id)

            assertEquals(expected = SuggestOutcome.NotReadable(detail = null), actual = outcome)
            assertTrue(actual = library.classifier.requests.isEmpty())
        }
    }
}
