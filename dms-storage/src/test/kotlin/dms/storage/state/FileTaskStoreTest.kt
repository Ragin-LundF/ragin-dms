package dms.storage.state

import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.SuggestedTag
import dms.domain.document.DocumentId
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.domain.task.TaskKey
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
            val full = ClassificationSuggestion(
                documentId = DocumentId(value = "doc-1"),
                model = "test-model",
                title = "Sample invoice",
                category = SuggestedCategory.New(name = "Invoices", parentId = CategoryId(value = "cat-1")),
                tags = listOf(
                    SuggestedTag(name = TagName.of(raw = "Tax"), isNew = false),
                    SuggestedTag(name = TagName.of(raw = "2024"), isNew = true),
                ),
                documentDate = LocalDate.parse("2023-12-24"),
                correspondent = "Example Supplier",
                fileName = "2023-12-24 Sample invoice.pdf",
                transcription = "Line one\nLine two",
                confidence = 0.75,
            )
            val existing = ClassificationSuggestion(
                documentId = DocumentId(value = "doc-2"),
                model = "test-model",
                category = SuggestedCategory.Existing(categoryId = CategoryId(value = "cat-1")),
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
            val first = ClassificationSuggestion(documentId = DocumentId(value = "doc-1"), model = "a")
            val second = ClassificationSuggestion(documentId = DocumentId(value = "doc-1"), model = "b")
            val other = ClassificationSuggestion(documentId = DocumentId(value = "doc-2"), model = "a")

            store.saveSuggestion(suggestion = first)
            store.saveSuggestion(suggestion = other)
            store.saveSuggestion(suggestion = second)
            store.removeSuggestion(documentId = DocumentId(value = "doc-2"))

            assertEquals(expected = listOf(second), actual = FileTaskStore(path = path).suggestions())
        }
    }

    @Test
    fun `dismissed tasks stay dismissed after a restart`() {
        runTest {
            val store = FileTaskStore(path = path)

            store.dismiss(key = TaskKey(value = "new-file:a.pdf"))
            store.dismiss(key = TaskKey(value = "new-file:a.pdf"))
            store.dismiss(key = TaskKey(value = "missing:b.pdf"))

            assertEquals(
                expected = setOf(TaskKey(value = "new-file:a.pdf"), TaskKey(value = "missing:b.pdf")),
                actual = FileTaskStore(path = path).dismissedKeys(),
            )
        }
    }
}
