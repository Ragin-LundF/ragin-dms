package dms.application.resolution

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.fake.storage.FakeFolders
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class KeepVersionTest {

    private val conflicted = RelativePath.of(raw = "notes/n.txt")

    @Test
    fun `nothing is replaced without confirmation`() {
        runTest {
            val library = TestLibrary()
            library.open()

            val outcome = library.keepVersion(path = conflicted, sourceFolderId = library.syncA.id, confirmed = false)

            assertEquals(expected = OperationOutcome.NotConfirmed, actual = outcome)
        }
    }

    @Test
    fun `the kept version replaces the others, which go to the trash`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = conflicted.value, content = "one")
            library.folders.put(folder = library.syncA, path = conflicted.value, content = "two")
            library.folders.put(folder = library.syncB, path = conflicted.value, content = "three")

            val outcome = library.keepVersion(path = conflicted, sourceFolderId = library.syncA.id, confirmed = true)

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "two",
                    actual = library.folders.content(folder = folder, path = conflicted.value),
                )
            }
            assertTrue(actual = library.primary.id to conflicted in library.folders.trashed)
            assertTrue(actual = library.syncB.id to conflicted in library.folders.trashed)
        }
    }

    @Test
    fun `accepting a modified document records its new content in the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "letters/a.pdf", content = "changed")

            val outcome = library.keepVersion(path = document.path, sourceFolderId = library.syncA.id, confirmed = true)

            val catalog = assertIs<OperationOutcome.Done>(value = outcome).catalog
            assertEquals(
                expected = FakeFolders.hashOf(content = "changed"),
                actual = catalog.documents.getValue(key = document.id).contentHash,
            )
            assertEquals(
                expected = "changed",
                actual = library.folders.content(folder = library.syncB, path = "letters/a.pdf"),
            )
        }
    }
}
