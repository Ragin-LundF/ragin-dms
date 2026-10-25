package dms.ui.ask

import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.ui.ask.model.AskEntryModel
import dms.ui.session.model.OperationLane
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class AskViewModelTest {

    @Test
    fun `a question is answered from the matching document with its source`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)
            app.ai.suggestion = "The bill came from Sample Energy [1]."

            ask.ask(question = "  Where is the energy bill?  ")
            app.idle()

            val entry = ask.state.value.entries.single()
            assertEquals(expected = "Where is the energy bill?", actual = entry.question)
            assertEquals(expected = "The bill came from Sample Energy [1].", actual = entry.answer)
            assertEquals(expected = listOf("energy-bill.txt"), actual = entry.citations.map { it.title })
            assertFalse(actual = entry.asking)
            assertEquals(
                expected = "Searched by words only; update the index to search by meaning",
                actual = entry.note,
            )
            assertTrue(actual = "energy-bill.txt" in app.ai.lastChatBody)
        }
    }

    @Test
    fun `a question nothing fits is answered honestly without asking the AI`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)

            ask.ask(question = "Which submarine schedules exist?")
            app.idle()

            val entry = ask.state.value.entries.single()
            assertNull(actual = entry.answer)
            assertEquals(expected = "Nothing in your documents fits this question.", actual = entry.note)
            assertEquals(expected = 0, actual = app.ai.chatRequests)
        }
    }

    @Test
    fun `a smart folder limits the documents the answer may come from`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            saveSmartFolder(app = app, name = "Letters", folder = "letters")
            val ask = AskViewModel(services = app.services, session = app.session)
            val scopes = ask.state.first { it.scopes.size == 2 }.scopes
            assertEquals(expected = listOf("Whole library", "Smart folder: Letters"), actual = scopes)

            ask.selectScope(index = 1)
            ask.ask(question = "Where is the energy bill?")
            app.idle()

            assertEquals(expected = 1, actual = ask.state.value.scopeIndex)
            assertEquals(
                expected = "Nothing in your documents fits this question.",
                actual = ask.state.value.entries.single().note,
            )
        }
    }

    @Test
    fun `a stopped AI server is named in the entry`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)
            app.ai.reachable = false

            ask.ask(question = "Where is the energy bill?")
            app.idle()

            val entry = ask.state.value.entries.single()
            assertNull(actual = entry.answer)
            assertTrue(actual = entry.note.orEmpty().startsWith(prefix = "The AI server does not answer"))
        }
    }

    @Test
    fun `without an open library the entry says so`() {
        UiTest.run { app ->
            val ask = AskViewModel(services = app.services, session = app.session)

            ask.ask(question = "Where is the energy bill?")
            app.idle()

            assertEquals(
                expected = listOf(AskEntryModel(question = "Where is the energy bill?", note = "No library is open.")),
                actual = ask.state.value.entries,
            )
        }
    }

    @Test
    fun `a blank question is ignored, and clear empties the conversation`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)

            ask.ask(question = "   ")
            assertEquals(expected = emptyList(), actual = ask.state.value.entries)
            assertTrue(actual = app.session.state.value.operations.isEmpty())

            ask.ask(question = "Which submarine schedules exist?")
            app.idle()
            ask.clear()

            assertEquals(expected = emptyList(), actual = ask.state.value.entries)
        }
    }

    @Test
    fun `while the AI lane is busy the question waits with a note`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)
            val gate = CompletableDeferred<Unit>()
            app.session.perform(label = "Sample AI work", lane = OperationLane.AI) { gate.await() }

            ask.ask(question = "Where is the energy bill?")

            val entry = ask.state.value.entries.single()
            assertEquals(expected = "Wait until \"Sample AI work\" finishes", actual = entry.note)
            assertFalse(actual = entry.asking)
            assertEquals(expected = "Sample AI work", actual = ask.state.value.aiBusy)
            gate.complete(value = Unit)
            app.idle()
            assertEquals(expected = 0, actual = app.ai.chatRequests)
        }
    }

    @Test
    fun `an index that cannot be built is reported as an error`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)

            ask.updateIndex()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertTrue(actual = message.text.startsWith(prefix = "The index could not be built: No embedding model"))
        }
    }

    @Test
    fun `an index with nothing to read is up to date`() {
        UiTest.run { app ->
            app.configure()
            val ask = AskViewModel(services = app.services, session = app.session)

            ask.updateIndex()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertFalse(actual = message.isError)
            assertEquals(expected = "The index is up to date; 0 documents were read anew", actual = message.text)
        }
    }

    private suspend fun libraryWithTwoFiles(app: TestApp) {
        app.write(file = app.primary.resolve("energy-bill.txt"), content = "alpha")
        app.write(file = app.primary.resolve("letters/garden-letter.txt"), content = "beta")
        app.configure(addNewFiles = true)
    }

    private suspend fun saveSmartFolder(app: TestApp, name: String, folder: String) {
        val search = SavedSearchModel(
            id = SavedSearchIdModel(value = "search-$name"),
            name = name,
            query = DocumentQueryModel(folder = RelativePathModel.of(raw = folder)),
        )
        app.services.documents.changeSettings(subject = "Smart folder $name") { settings ->
            settings.copy(savedSearches = settings.savedSearches + search)
        }
    }

    @Test
    fun `a cancelled question does not stay in the asking state`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val ask = AskViewModel(services = app.services, session = app.session)
            app.ai.answerAtOnce = 0

            ask.ask(question = "Where is the energy bill?")
            app.session.state.first { it.aiBusy != null }
            app.session.cancel(lane = OperationLane.AI)
            app.idle()

            assertEquals(expected = "Asking the library: cancelled", actual = app.session.state.value.message?.text)
            val entry = ask.state.first { state -> state.entries.none { it.asking } }.entries.single()
            assertEquals(expected = "Stopped before an answer", actual = entry.note)
        }
    }
}
