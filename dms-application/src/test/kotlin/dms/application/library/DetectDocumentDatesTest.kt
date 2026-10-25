package dms.application.library

import dms.application.TestLibrary
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DetectDocumentDatesTest {

    @Test
    fun `new files get the date of their name, else of their creation, else none`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "15.03.2024 Sample.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "created.pdf", content = "beta")
            library.folders.put(folder = library.primary, path = "unknown.pdf", content = "gamma")
            library.folders.creationDays[RelativePathModel.of(raw = "created.pdf")] = LocalDate.parse("2019-06-01")

            library.scanned(addNewFiles = true)

            val named = dateOf(library = library, path = "15.03.2024 Sample.pdf")
            assertEquals(expected = LocalDate.parse("2024-03-15"), actual = named)
            val created = dateOf(library = library, path = "created.pdf")
            assertEquals(expected = LocalDate.parse("2019-06-01"), actual = created)
            assertNull(actual = dateOf(library = library, path = "unknown.pdf"))
        }
    }

    @Test
    fun `documents without a date get one later, dates already set stay`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "2021-02-03 Sample.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "other.pdf", content = "beta")
            library.scanned(addNewFiles = true)
            library.folders.creationDays[RelativePathModel.of(raw = "other.pdf")] = LocalDate.parse("2018-01-01")

            val dated = library.detectDocumentDates()

            assertEquals(expected = 1, actual = dated)
            val other = dateOf(library = library, path = "other.pdf")
            assertEquals(expected = LocalDate.parse("2018-01-01"), actual = other)
            val named = dateOf(library = library, path = "2021-02-03 Sample.pdf")
            assertEquals(expected = LocalDate.parse("2021-02-03"), actual = named)
        }
    }

    private fun dateOf(library: TestLibrary, path: String): LocalDate? {
        return library.catalog.findByPath(path = RelativePathModel.of(raw = path))?.metadata?.documentDate
    }
}
