package dms.application.search

import dms.application.TestLibrary
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
}
