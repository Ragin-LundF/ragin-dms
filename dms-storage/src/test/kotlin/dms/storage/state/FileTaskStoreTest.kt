package dms.storage.state

import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedTagModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.task.model.TaskKeyModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.time.LocalDate
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FileTaskStoreTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val path: Path = temp.resolve("tasks.yaml")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `suggestions survive a restart with every field`() {
        runTest {
            val full = ClassificationSuggestionModel(
                documentId = DocumentIdModel(value = "doc-1"),
                model = "test-model",
                title = "Sample invoice",
                category = SuggestedCategory.New(name = "Invoices", parentId = CategoryIdModel(value = "cat-1")),
                tags = listOf(
                    SuggestedTagModel(name = TagNameModel.of(raw = "Tax"), isNew = false),
                    SuggestedTagModel(name = TagNameModel.of(raw = "2024"), isNew = true),
                ),
                documentDate = LocalDate.parse("2023-12-24"),
                correspondent = "Example Supplier",
                fileName = "2023-12-24 Sample invoice.pdf",
                transcription = "Line one\nLine two",
                confidence = 0.75,
            )
            val existing = ClassificationSuggestionModel(
                documentId = DocumentIdModel(value = "doc-2"),
                model = "test-model",
                category = SuggestedCategory.Existing(categoryId = CategoryIdModel(value = "cat-1")),
            )
            val store = FileTaskStore(path = path)

            store.saveSuggestion(suggestion = full)
            store.saveSuggestion(suggestion = existing)

            assertEquals(expected = listOf(full, existing), actual = FileTaskStore(path = path).suggestions())
        }
    }

    @Test
    fun `a new suggestion replaces the previous one of the same document`() {
        runTest {
            val store = FileTaskStore(path = path)
            val first = ClassificationSuggestionModel(documentId = DocumentIdModel(value = "doc-1"), model = "a")
            val second = ClassificationSuggestionModel(documentId = DocumentIdModel(value = "doc-1"), model = "b")
            val other = ClassificationSuggestionModel(documentId = DocumentIdModel(value = "doc-2"), model = "a")

            store.saveSuggestion(suggestion = first)
            store.saveSuggestion(suggestion = other)
            store.saveSuggestion(suggestion = second)
            store.removeSuggestion(documentId = DocumentIdModel(value = "doc-2"))

            assertEquals(expected = listOf(second), actual = FileTaskStore(path = path).suggestions())
        }
    }

    @Test
    fun `dismissed tasks stay dismissed after a restart`() {
        runTest {
            val store = FileTaskStore(path = path)

            store.dismiss(key = TaskKeyModel(value = "new-file:a.pdf"))
            store.dismiss(key = TaskKeyModel(value = "new-file:a.pdf"))
            store.dismiss(key = TaskKeyModel(value = "missing:b.pdf"))

            assertEquals(
                expected = setOf(TaskKeyModel(value = "new-file:a.pdf"), TaskKeyModel(value = "missing:b.pdf")),
                actual = FileTaskStore(path = path).dismissedKeys(),
            )
        }
    }
}
