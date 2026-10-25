package dms.ui.document

import dms.domain.document.model.DocumentModel
import dms.ui.document.model.InspectorTab
import dms.ui.document.suggestion.SuggestionValues
import dms.ui.support.SampleImages
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class InspectorViewModelTest {

    @Test
    fun `the chosen tab stays, and Pages shows Details for a file that is no PDF`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.txt")
            val inspector = inspectorOf(app = app)
            val pdf = documents.getValue(key = "a.pdf")
            val text = documents.getValue(key = "b.txt")

            val initial = inspector.tab.value
            inspector.selectTab(tab = InspectorTab.PAGES)

            assertEquals(expected = InspectorTab.DETAILS, actual = initial)
            assertEquals(expected = InspectorTab.PAGES, actual = inspector.tab.value)
            assertEquals(
                expected = InspectorTab.PAGES,
                actual = inspector.shownTab(selected = InspectorTab.PAGES, document = pdf),
            )
            assertEquals(
                expected = InspectorTab.DETAILS,
                actual = inspector.shownTab(selected = InspectorTab.PAGES, document = text),
            )
            assertEquals(expected = setOf(InspectorTab.PAGES), actual = inspector.disabledTabs(document = text).keys)
            assertEquals(expected = emptyMap(), actual = inspector.disabledTabs(document = pdf))
        }
    }

    @Test
    fun `a pending suggestion shows for its document, opens the review, and dismissing it rejects it`() {
        UiTest.run { app ->
            val inspector = inspectorOf(app = app)
            val document = libraryWithOneImage(app = app)
            app.ai.suggestion = """{"title":"Sample bill","correspondent":"Sample Energy","tags":["Tax"]}"""
            inspector.classifyWithAi(document = document)
            app.idle()
            inspector.review.close()

            val feed = inspector.feed.state.first { document.id in it.suggestions }
            val pending = feed.suggestions.getValue(key = document.id)
            val values = SuggestionValues.of(suggestion = pending, catalog = SampleLibrary.catalog(app = app))
            inspector.reviewSuggestion(suggestion = pending)
            val review = assertNotNull(actual = inspector.review.state.value)
            inspector.review.close()
            inspector.dismissSuggestion(document = document)
            app.idle()
            val after = inspector.feed.state.first { document.id !in it.suggestions }

            assertTrue(actual = "title: Sample bill" in values)
            assertTrue(actual = "correspondent: Sample Energy" in values)
            assertEquals(expected = "Sample bill", actual = review.draft.title.value)
            assertTrue(actual = after.suggestions.isEmpty())
            assertNull(actual = SampleLibrary.document(app = app, id = document.id).metadata.title)
        }
    }

    private fun inspectorOf(app: TestApp): InspectorViewModel {
        return InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
    }

    private suspend fun libraryWithOneImage(app: TestApp): DocumentModel {
        app.configure()
        val library = app.libraryViewModel()
        val source = SampleImages.write(file = app.root.resolve("outside/scan-7.png"))
        app.platform.nextFiles = listOf(source.toString())
        library.addFilesToTopLevel()
        app.idle()
        return library.state.first { it.documents.isNotEmpty() }.documents.single().document
    }
}
