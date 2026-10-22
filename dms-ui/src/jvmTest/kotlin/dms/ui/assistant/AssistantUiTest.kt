package dms.ui.assistant

import dms.application.port.assistant.model.ChatRole
import dms.domain.document.model.DocumentModel
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.document.InspectorViewModel
import dms.ui.support.SampleImages
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class AssistantUiTest {

    @Test
    fun `the summary of a document is shown`() {
        UiTest.run { app ->
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = "A sample bill from Sample Energy."

            inspector.summary.open(document = document)

            val summary = inspector.summary.state.first { it?.working == false }
            assertEquals(expected = "A sample bill from Sample Energy.", actual = summary?.text)
            assertTrue(actual = "summarize" in app.ai.lastChatBody)
        }
    }

    @Test
    fun `tags from the AI open a review with the tags only`() {
        UiTest.run { app ->
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = "Tags: Invoice, Electricity"

            inspector.tagWithAi(document = document)
            app.idle()

            val review = assertNotNull(actual = inspector.review.state.value)
            assertEquals(expected = "Invoice, Electricity", actual = review.draft.tags.value)
            assertNull(actual = review.suggestion.title)
        }
    }

    @Test
    fun `a question is answered, and a failed one goes back into the input`() {
        UiTest.run { app ->
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = "On 15 March 2024."
            inspector.chat.open(document = document)

            inspector.chat.send(question = "When was it issued?")
            val answered = inspector.chat.state.first { it?.waiting == false }
            app.ai.reachable = false
            inspector.chat.send(question = "Who sent it?")
            val failed = inspector.chat.state.first { it?.problem != null }

            assertEquals(
                expected = listOf(ChatRole.USER to "When was it issued?", ChatRole.ASSISTANT to "On 15 March 2024."),
                actual = answered?.turns?.map { it.role to it.text },
            )
            assertEquals(expected = 2, actual = failed?.turns?.size)
            assertEquals(expected = "Who sent it?", actual = failed?.input)
        }
    }

    @Test
    fun `the dashboard tags every untagged document for review`() {
        UiTest.run { app ->
            val dashboard = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            libraryWithOneImage(app = app)
            app.ai.suggestion = "Invoice"
            assertEquals(expected = 1, actual = dashboard.state.first { it.tagCandidates > 0 }.tagCandidates)

            dashboard.aiBatches.tagAll()
            app.idle()

            assertEquals(expected = 0, actual = dashboard.state.first { it.tagCandidates == 0 }.tagCandidates)
            assertEquals(expected = 1, actual = app.ai.chatRequests)
        }
    }

    @Test
    fun `each tag suggestion can be reviewed while the run goes on`() {
        UiTest.run { app ->
            val dashboard = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            libraryWithImages(app = app, names = listOf("scan-7.png", "scan-8.png"))
            app.ai.suggestion = "Invoice"
            app.ai.answerAtOnce = 1

            dashboard.aiBatches.tagAll()

            val running = dashboard.state.first { state -> state.groups.any { it.group == TaskGroup.SUGGESTIONS } }
            assertNotNull(actual = running.aiBusy)
            assertNull(actual = running.busy)
            val suggestions = running.groups.single { it.group == TaskGroup.SUGGESTIONS }
            suggestions.rows.first().buttons.single { it.label == "Review" }.onClick()
            dashboard.review.approve()
            assertNull(actual = dashboard.review.state.value)
            app.ai.release.complete(value = Unit)
            app.idle()
            assertEquals(expected = 2, actual = app.ai.chatRequests)
        }
    }

    private suspend fun libraryWithOneImage(app: TestApp): DocumentModel {
        return libraryWithImages(app = app, names = listOf("scan-7.png")).single()
    }

    private suspend fun libraryWithImages(app: TestApp, names: List<String>): List<DocumentModel> {
        app.configure()
        val library = app.libraryViewModel()
        app.platform.nextFiles = names.map { SampleImages.write(file = app.root.resolve("outside/$it")).toString() }
        library.addFilesToTopLevel()
        app.idle()
        return library.state.first { it.documents.size == names.size }.documents.map { it.document }
    }
}
