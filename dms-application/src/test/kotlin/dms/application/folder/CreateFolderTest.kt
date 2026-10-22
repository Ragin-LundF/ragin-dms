package dms.application.folder

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class CreateFolderTest {

    @Test
    fun `a folder is created in every library folder`() {
        runTest {
            val library = TestLibrary()
            library.open()

            assertIs<OperationOutcome.Done>(value = library.createFolder(path = RelativePathModel.of(raw = "archive")))

            library.allFolders.forEach { folder ->
                assertTrue(actual = library.folders.hasDirectory(folder = folder, path = "archive"))
            }
        }
    }

    @Test
    fun `a folder name that differs only by case from an existing one is refused`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.createFolder(path = RelativePathModel.of(raw = "archive"))

            val outcome = library.createFolder(path = RelativePathModel.of(raw = "Archive"))

            assertEquals(expected = OperationOutcome.NameTaken, actual = outcome)
        }
    }
}
