package dms.application.classification

import dms.application.TestLibrary
import dms.application.classification.model.SuggestOutcome
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ClassifyAllNewTest {

    @Test
    fun `every readable document without a classification gets a suggestion once`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "inbox/a.pdf", content = "a")
            library.add(path = "inbox/b.png", content = "b")
            library.add(path = "inbox/c.txt", content = "c")

            val progress = library.classifyAllNew().toList()
            val again = library.classifyAllNew().toList()

            assertEquals(expected = listOf(1, 2), actual = progress.map { it.done })
            assertEquals(expected = listOf(2, 2), actual = progress.map { it.total })
            assertTrue(actual = progress.all { it.outcome is SuggestOutcome.Suggested })
            assertTrue(actual = again.isEmpty())
        }
    }
}
