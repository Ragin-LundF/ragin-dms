package dms.ui.review

import dms.domain.document.model.DocumentModel
import dms.ui.document.QuickLookViewModel
import dms.ui.support.SampleImages
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ReviewControllerTest {

    private val answer = """
        {"title":"Sample bill","documentDate":"2024-03-15","correspondent":"Sample Energy",
         "tags":["Tax"],"fileName":"2024-03-15 Sample Energy - Bill.png"}
    """.trimIndent()

    @Test
    fun `asking the AI opens a review and changes nothing yet`() {
        UiTest.run { app ->
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = answer

            quickLookViewModel.classifyWithAi(document = document)
            app.idle()

            val review = assertNotNull(actual = quickLookViewModel.review.state.value)
            assertEquals(expected = "Sample bill", actual = review.draft.title.value)
            assertEquals(expected = "2024-03-15 Sample Energy - Bill.png", actual = review.draft.fileName.value)
            assertFalse(actual = review.draft.fileName.accepted)
            assertTrue(actual = Files.exists(app.primary.resolve("scan-7.png")))
            assertEquals(expected = 1, actual = app.ai.chatRequests)
        }
    }

    @Test
    fun `an approved review applies the edited values and renames the file everywhere`() {
        UiTest.run { app ->
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = answer
            quickLookViewModel.classifyWithAi(document = document)
            app.idle()
            val draft = assertNotNull(actual = quickLookViewModel.review.state.value).draft

            val edited = draft.copy(
                title = draft.title.copy(value = "Edited bill"),
                fileName = draft.fileName.copy(accepted = true),
            )
            quickLookViewModel.review.update(draft = edited)
            quickLookViewModel.review.approve()
            app.idle()

            assertNull(actual = quickLookViewModel.review.state.value)
            val filed = app.services.library.catalog.value?.documents?.get(document.id)
            assertEquals(expected = "Edited bill", actual = filed?.metadata?.title)
            assertEquals(expected = LocalDate.parse("2024-03-15"), actual = filed?.metadata?.documentDate)
            assertEquals(expected = "2024-03-15 Sample Energy - Bill.png", actual = filed?.path?.value)
            assertTrue(actual = Files.exists(app.primary.resolve("2024-03-15 Sample Energy - Bill.png")))
            assertTrue(actual = Files.exists(app.sync.resolve("2024-03-15 Sample Energy - Bill.png")))
        }
    }

    @Test
    fun `a stopped AI server is reported and no review opens`() {
        UiTest.run { app ->
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            val document = libraryWithOneImage(app = app)
            app.ai.reachable = false

            quickLookViewModel.classifyWithAi(document = document)
            app.idle()

            assertNull(actual = quickLookViewModel.review.state.value)
            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertTrue(actual = "does not answer" in message.text)
        }
    }

    private suspend fun libraryWithOneImage(app: TestApp): DocumentModel {
        app.configure()
        val library = app.libraryViewModel()
        val source = SampleImages.write(file = app.root.resolve("outside/scan-7.png"))
        app.platform.nextFiles = listOf(source.toString())
        library.files.addFilesToTopLevel()
        app.idle()
        return library.state.first { it.documents.isNotEmpty() }.documents.single().document
    }
}
