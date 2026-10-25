package dms.application.search

import dms.application.TestLibrary
import dms.application.search.model.FindOutcome
import dms.application.search.model.SearchOutcome
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class SearchContentsTest {

    private suspend fun found(library: TestLibrary, query: String): Set<DocumentIdModel> {
        return assertIs<SearchOutcome.Found>(value = library.searchContents(query = query)).documentIds
    }

    private suspend fun matched(library: TestLibrary, query: DocumentQueryModel): List<DocumentModel> {
        return assertIs<FindOutcome.Found>(value = library.findDocuments(query = query)).documents
    }

    @Test
    fun `nothing is searched while no library is open`() {
        runTest {
            val library = TestLibrary()

            assertEquals(expected = SearchOutcome.LibraryNotOpen, actual = library.searchContents(query = "a"))
            assertEquals(
                expected = FindOutcome.LibraryNotOpen,
                actual = library.findDocuments(query = DocumentQueryModel.ALL),
            )
        }
    }

    @Test
    fun `documents are found by the text of their file, ignoring case`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bills/power.pdf", content = "a")
            library.add(path = "bills/water.pdf", content = "b")
            library.reader.texts = mapOf("power.pdf" to "Sample Energy Ltd, meter reading 4711")

            assertEquals(expected = setOf(bill.id), actual = found(library = library, query = "  METER reading "))
            assertEquals(expected = emptySet(), actual = found(library = library, query = "insurance"))
        }
    }

    @Test
    fun `the text the AI transcribed is found too`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scans/receipt.png", content = "a")
            library.folders.write(folder = library.primary, documentId = scan.id, text = "Sample Bakery receipt")

            assertEquals(expected = setOf(scan.id), actual = found(library = library, query = "bakery"))
        }
    }

    @Test
    fun `each file is read once until it changes`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "a.pdf", content = "a")
            library.add(path = "b.pdf", content = "b")

            library.searchContents(query = "first")
            library.searchContents(query = "second")

            assertEquals(expected = 2, actual = library.reader.textReads)
        }
    }

    @Test
    fun `an empty query finds nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "a.pdf", content = "a")

            assertEquals(expected = emptySet(), actual = found(library = library, query = " "))
        }
    }

    @Test
    fun `a note counts as text of its document`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bills/power.pdf", content = "a")
            library.store.commit { it.setNote(documentId = bill.id, note = "Disputed with the supplier") }

            assertEquals(expected = setOf(bill.id), actual = found(library = library, query = "disputed"))
        }
    }

    @Test
    fun `a query filters by its conditions and then by the content term`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val power = library.add(path = "bills/power.pdf", content = "a")
            library.add(path = "bills/water.pdf", content = "b")
            library.add(path = "letters/power-note.pdf", content = "c")
            library.reader.texts = mapOf("power.pdf" to "meter reading", "power-note.pdf" to "meter reading")

            val inBills = DocumentQueryModel(folder = RelativePathModel.of(raw = "bills"), contentText = "METER")

            assertEquals(expected = listOf(power), actual = matched(library = library, query = inBills))
            assertEquals(
                expected = 3,
                actual = matched(library = library, query = DocumentQueryModel.ALL).size,
            )
        }
    }
}
