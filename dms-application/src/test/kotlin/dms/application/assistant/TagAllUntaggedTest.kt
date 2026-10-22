package dms.application.assistant

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.domain.document.model.DocumentIdModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class TagAllUntaggedTest {

    @Test
    fun `only readable documents without tags are tagged, each once`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "inbox/a.pdf", content = "a")
            val tagged = library.add(path = "inbox/b.pdf", content = "b")
            library.add(path = "inbox/c.txt", content = "c")
            val tags = setOf(TagNameModel.of(raw = "Home"))
            library.editCatalog(edit = CatalogEdit.AddTags(documentId = tagged.id, tags = tags))

            val progress = library.tagAllUntagged().toList()
            val again = library.tagAllUntagged().toList()

            val paths = progress.map { pathOf(library = library, id = it.documentId) }
            assertEquals(expected = listOf("inbox/a.pdf"), actual = paths)
            assertTrue(actual = again.isEmpty())
        }
    }

    private fun pathOf(library: TestLibrary, id: DocumentIdModel): String {
        return library.catalog.documents.getValue(id).path.value
    }
}
