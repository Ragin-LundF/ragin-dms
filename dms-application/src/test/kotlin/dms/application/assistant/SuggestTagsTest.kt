package dms.application.assistant

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.application.classification.model.SuggestOutcome
import dms.application.fake.ai.FakeAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.domain.taxonomy.TagName
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class SuggestTagsTest {

    private val tax = TagName.of(raw = "Tax")
    private val energy = TagName.of(raw = "Energy")

    @Test
    fun `suggested tags wait for review, without the ones the document has`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/bill.pdf", content = "a")
            library.editCatalog(edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(tax)))
            library.assistant.tags = AssistantAnswer.Tags(tags = listOf(tax, energy), model = FakeAssistant.MODEL)

            val outcome = library.suggestTags(documentId = document.id)

            val suggestion = assertIs<SuggestOutcome.Suggested>(value = outcome).suggestion
            assertEquals(expected = listOf(energy), actual = suggestion.tags.map { it.name })
            assertTrue(actual = suggestion.tags.single().isNew)
            assertEquals(expected = listOf(suggestion), actual = library.tasks.suggestions())
        }
    }

    @Test
    fun `no new tags is answered but not stored`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/bill.pdf", content = "a")
            library.assistant.tags = AssistantAnswer.Tags(tags = emptyList(), model = FakeAssistant.MODEL)

            val outcome = library.suggestTags(documentId = document.id)

            assertTrue(actual = assertIs<SuggestOutcome.Suggested>(value = outcome).suggestion.tags.isEmpty())
            assertTrue(actual = library.tasks.suggestions().isEmpty())
        }
    }

    @Test
    fun `a file the AI does not read is not sent`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/notes.txt", content = "a")

            val outcome = library.suggestTags(documentId = document.id)

            assertIs<SuggestOutcome.NotReadable>(value = outcome)
            assertTrue(actual = library.assistant.requests.isEmpty())
        }
    }
}
