package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.fake.storage.FakeFolders
import dms.domain.change.DetectedChange
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull

class AdoptNewFileTest {

    @Test
    fun `a file found in one sync folder is catalogued and copied into every folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.syncA, path = "inbox/scan.pdf", content = "scan")
            val scan = assertNotNull(actual = library.scanned())
            val newFile = scan.changes.filterIsInstance<DetectedChange.NewFile>().single()

            val outcome = library.adoptNewFile(change = newFile)

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "scan",
                    actual = library.folders.content(folder = folder, path = "inbox/scan.pdf"),
                )
            }
            val document = assertNotNull(
                actual = library.catalog.findByPath(path = RelativePathModel.of(raw = "inbox/scan.pdf")),
            )
            assertEquals(expected = FakeFolders.hashOf(content = "scan"), actual = document.contentHash)
        }
    }
}
