package dms.application.library

import dms.application.TestLibrary
import dms.application.library.model.outcome.OpenLibraryOutcome
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class OpenLibraryTest {

    private val libraryId = LibraryIdModel(value = "library-1")

    @Test
    fun `a new library starts with an empty catalog saved to every folder`() {
        runTest {
            val library = TestLibrary()

            val catalog = library.open()

            assertTrue(actual = catalog.documents.isEmpty())
            library.allFolders.forEach { folder ->
                assertEquals(expected = catalog, actual = library.folders.catalogs[folder.id])
            }
        }
    }

    @Test
    fun `catalog copies and conflict copies are merged and saved to every folder`() {
        runTest {
            val library = TestLibrary()
            val empty = CatalogModel.empty(libraryId = libraryId)
            val first = TestLibrary.document(id = "d1", path = "letters/a.pdf", content = "a")
            val second = TestLibrary.document(id = "d2", path = "letters/b.pdf", content = "b")
            val third = TestLibrary.document(id = "d3", path = "letters/c.pdf", content = "c")
            library.folders.catalogs[library.primary.id] = applied(update = empty.addDocument(document = first))
            library.folders.catalogs[library.syncA.id] = applied(update = empty.addDocument(document = second))
            library.folders.conflictCopies[library.primary.id] =
                mutableListOf(applied(update = empty.addDocument(document = third)))

            val catalog = library.open()

            assertEquals(expected = setOf(first.id, second.id, third.id), actual = catalog.documents.keys)
            library.allFolders.forEach { folder ->
                assertEquals(expected = catalog, actual = library.folders.catalogs[folder.id])
            }
            assertTrue(actual = library.folders.conflictCopies[library.primary.id].isNullOrEmpty())
        }
    }

    @Test
    fun `a folder holding another library is reported and not overwritten`() {
        runTest {
            val library = TestLibrary()
            val foreign = CatalogModel.empty(libraryId = LibraryIdModel(value = "other-library"))
            library.folders.catalogs[library.primary.id] = CatalogModel.empty(libraryId = libraryId)
            library.folders.catalogs[library.syncB.id] = foreign

            val outcome = assertIs<OpenLibraryOutcome.Opened>(value = library.openLibrary())

            assertEquals(expected = setOf(library.syncB.id), actual = outcome.foreignFolders)
            assertEquals(expected = foreign, actual = library.folders.catalogs[library.syncB.id])
            assertEquals(expected = libraryId, actual = outcome.catalog.libraryId)

            library.add(path = "letters/a.pdf", content = "alpha")

            assertEquals(expected = foreign, actual = library.folders.catalogs[library.syncB.id])
            assertTrue(actual = library.folders.paths(folder = library.syncB).isEmpty())
        }
    }

    @Test
    fun `an unreachable sync folder does not keep the library from opening`() {
        runTest {
            val library = TestLibrary()
            library.folders.unreachable += library.syncA.id

            val outcome = assertIs<OpenLibraryOutcome.Opened>(value = library.openLibrary())

            assertEquals(expected = setOf(library.syncA.id), actual = outcome.unreachableFolders)
        }
    }

    @Test
    fun `an unreachable primary folder opens nothing`() {
        runTest {
            val library = TestLibrary()
            library.folders.unreachable += library.primary.id

            assertIs<OpenLibraryOutcome.PrimaryFailed>(value = library.openLibrary())
            assertEquals(expected = null, actual = library.store.catalog.value)
        }
    }

    private fun applied(update: CatalogUpdate): CatalogModel {
        return assertIs<CatalogUpdate.Applied>(value = update).catalog
    }
}
