package dms.application.assistant

import dms.application.TestLibrary
import dms.application.assistant.model.AnswerOutcome
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class SummarizeDocumentTest {

    @Test
    fun `a summary is answered for a readable document`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/bill.pdf", content = "a")

            val outcome = library.summarizeDocument(documentId = document.id)

            val answered = assertIs<AnswerOutcome.Answered>(value = outcome)
            assertEquals(expected = "A synthetic summary", actual = answered.text)
            assertEquals(expected = "bill.pdf", actual = library.assistant.requests.single().fileName)
        }
    }
}
