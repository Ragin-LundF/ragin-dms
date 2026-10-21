package dms.application.folder

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.catalog.CatalogRejection
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class MoveFolderTest {

    private val letters = RelativePath.of(raw = "letters")

    @Test
    fun `a folder moves with its documents in every folder and in the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")
            library.add(path = "letters/b.pdf", content = "beta")

            val outcome = library.moveFolder(from = letters, to = RelativePath.of(raw = "archive/letters"))

            assertIs<OperationOutcome.Done>(value = outcome)
            val expected = setOf("archive/letters/a.pdf", "archive/letters/b.pdf")
            library.allFolders.forEach { folder ->
                assertEquals(expected = expected, actual = library.folders.paths(folder = folder))
            }
            assertEquals(expected = expected, actual = library.catalog.documents.values.map { it.path.value }.toSet())
        }
    }

    @Test
    fun `a folder cannot move into itself`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")

            val outcome = library.moveFolder(from = letters, to = RelativePath.of(raw = "letters/inner"))

            assertEquals(
                expected = OperationOutcome.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE),
                actual = outcome,
            )
        }
    }
}
