package dms.application.resolution

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class KeepBothVersionsTest {

    @Test
    fun `one version is renamed and copied into every folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "notes/n.txt", content = "one")
            library.folders.put(folder = library.syncA, path = "notes/n.txt", content = "two")

            val outcome = library.keepBothVersions(
                path = RelativePath.of(raw = "notes/n.txt"),
                folderId = library.syncA.id,
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "two",
                    actual = library.folders.content(folder = folder, path = "notes/n (2).txt"),
                )
            }
            assertEquals(
                expected = "one",
                actual = library.folders.content(folder = library.primary, path = "notes/n.txt"),
            )
        }
    }
}
