package dms.application.assistant

import dms.application.TestLibrary
import dms.application.assistant.model.AnswerOutcome
import dms.application.port.ai.model.DocumentContent
import dms.application.port.assistant.model.ChatRole
import dms.application.port.assistant.model.ChatTurnModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class AskAboutDocumentTest {

    @Test
    fun `the conversation goes with the question, and the document is read once`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "inbox/bill.pdf", content = "a")
            val first = listOf(ChatTurnModel(role = ChatRole.USER, text = "Who sent it?"))
            val second = first + listOf(
                ChatTurnModel(role = ChatRole.ASSISTANT, text = "Sample Energy"),
                ChatTurnModel(role = ChatRole.USER, text = "When?"),
            )

            library.askAboutDocument(documentId = document.id, conversation = first)
            library.reader.content = DocumentContent.Text(text = "changed, but the document is unchanged")
            val outcome = library.askAboutDocument(documentId = document.id, conversation = second)

            val answered = assertIs<AnswerOutcome.Answered>(value = outcome)
            assertEquals(expected = "A synthetic answer", actual = answered.text)
            assertEquals(expected = listOf(first, second), actual = library.assistant.conversations)
            assertEquals(
                expected = listOf("synthetic text", "synthetic text"),
                actual = library.assistant.requests.map { (it.content as DocumentContent.Text).text },
            )
        }
    }
}
