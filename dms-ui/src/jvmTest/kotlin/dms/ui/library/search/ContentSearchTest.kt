package dms.ui.library.search

import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals

class ContentSearchTest {

    @Test
    fun `the search finds a document by its text until the switch is turned off`() {
        UiTest.run { app ->
            app.configure()
            val library = app.libraryViewModel()
            val source = app.write(file = app.root.resolve("outside/receipt-3.pdf"), content = "alpha")
            app.platform.nextFiles = listOf(source.toString())
            library.addFilesToTopLevel()
            app.idle()
            val document = library.state.first { it.documents.isNotEmpty() }.documents.single().document
            val transcription = app.primary.resolve(".dms-text/${document.id.value}.txt")
            app.write(file = transcription, content = "Sample Bakery, 2 rolls")

            library.search(query = "bakery")
            val found = library.state.first { !it.contentSearch.searching && it.documents.isNotEmpty() }
            library.contentSearch.enable(on = false)
            val switchedOff = library.state.first { !it.contentSearch.enabled }

            assertEquals(expected = document.id, actual = found.documents.single().document.id)
            assertEquals(expected = 0, actual = switchedOff.documents.size)
        }
    }
}
