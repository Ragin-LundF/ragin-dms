package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.fake.storage.FakeFolders
import dms.application.port.storage.model.StorageFailure
import dms.domain.library.FileNameViolation
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class AddDocumentsTest {

    private val letters = RelativePathModel.of(raw = "letters")

    @Test
    fun `a file is copied into every folder and catalogued`() {
        runTest {
            val library = TestLibrary()
            library.open()

            val document = library.add(path = "letters/a.pdf", content = "alpha")

            assertEquals(expected = FakeFolders.hashOf(content = "alpha"), actual = document.contentHash)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "alpha",
                    actual = library.folders.content(folder = folder, path = "letters/a.pdf"),
                )
                assertEquals(expected = library.catalog, actual = library.folders.catalogs[folder.id])
            }
        }
    }

    @Test
    fun `a name that is taken, ignoring case, gets a suffix`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")
            val source = library.folders.external(location = "/outside/A.pdf", content = "beta")

            val added = library.addDocuments(files = listOf(source), targetFolder = letters).single()

            assertEquals(expected = RelativePathModel.of(raw = "letters/A (2).pdf"), actual = added.path)
            assertEquals(
                expected = "alpha",
                actual = library.folders.content(folder = library.syncA, path = "letters/a.pdf"),
            )
        }
    }

    @Test
    fun `a name the file name policy refuses is not added`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val source = library.folders.external(location = "/outside/con.pdf", content = "x")

            val added = library.addDocuments(files = listOf(source), targetFolder = letters).single()

            assertEquals(
                expected = OperationOutcome.InvalidName(violation = FileNameViolation.RESERVED_NAME),
                actual = added.outcome,
            )
            assertTrue(actual = library.folders.paths(folder = library.primary).isEmpty())
        }
    }

    @Test
    fun `an unreachable sync folder is journaled and reported as pending`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id
            val source = library.folders.external(location = "/outside/a.pdf", content = "alpha")

            val added = library.addDocuments(files = listOf(source), targetFolder = letters).single()

            val done = assertIs<OperationOutcome.Done>(value = added.outcome)
            assertEquals(expected = setOf(library.syncB.id), actual = done.pendingFolders)
            assertEquals(expected = 2, actual = library.journal.pending().size)
            assertEquals(
                expected = "alpha",
                actual = library.folders.content(folder = library.syncA, path = "letters/a.pdf"),
            )
        }
    }

    @Test
    fun `an unreachable primary folder changes nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.primary.id
            val source = library.folders.external(location = "/outside/a.pdf", content = "alpha")

            val added = library.addDocuments(files = listOf(source), targetFolder = letters).single()

            val failed = assertIs<OperationOutcome.StorageFailed>(value = added.outcome)
            assertEquals(expected = StorageFailure.FOLDER_UNREACHABLE, actual = failed.failure.reason)
            assertTrue(actual = library.folders.paths(folder = library.syncA).isEmpty())
            assertTrue(actual = library.catalog.documents.isEmpty())
        }
    }

    @Test
    fun `content the library already has is added and reported as a duplicate`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val existing = library.add(path = "letters/a.pdf", content = "alpha")
            val source = library.folders.external(location = "/outside/b.pdf", content = "alpha")

            val added = library.addDocuments(files = listOf(source), targetFolder = letters).single()

            assertIs<OperationOutcome.Done>(value = added.outcome)
            assertEquals(expected = listOf(existing), actual = added.duplicates)
        }
    }
}
