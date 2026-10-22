package dms.application.search

import dms.application.TestLibrary
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class SearchContentsTest {

    @Test
    fun `documents are found by the text of their file, ignoring case`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bills/power.pdf", content = "a")
            library.add(path = "bills/water.pdf", content = "b")
            library.reader.texts = mapOf("power.pdf" to "Sample Energy Ltd, meter reading 4711")

            assertEquals(expected = setOf(bill.id), actual = library.searchContents(query = "  METER reading "))
            assertEquals(expected = emptySet(), actual = library.searchContents(query = "insurance"))
        }
    }

    @Test
    fun `the text the AI transcribed is found too`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scans/receipt.png", content = "a")
            library.folders.write(folder = library.primary, documentId = scan.id, text = "Sample Bakery receipt")

            assertEquals(expected = setOf(scan.id), actual = library.searchContents(query = "bakery"))
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

            assertEquals(expected = emptySet(), actual = library.searchContents(query = " "))
        }
    }

    @Test
    fun `a note counts as text of its document`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bills/power.pdf", content = "a")
            library.store.commit { it.setNote(documentId = bill.id, note = "Disputed with the supplier") }

            assertEquals(expected = setOf(bill.id), actual = library.searchContents(query = "disputed"))
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

            assertEquals(expected = listOf(power), actual = library.searchContents.search(query = inBills))
            assertEquals(
                expected = 3,
                actual = library.searchContents.search(query = DocumentQueryModel.ALL).size,
            )
        }
    }
}
