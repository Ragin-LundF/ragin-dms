package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.library.FileNameViolation
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class MoveDocumentTest {

    @Test
    fun `a document is moved in every folder and in the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            val target = RelativePath.of(raw = "archive/2024/a.pdf")

            assertIs<OperationOutcome.Done>(value = library.moveDocument(documentId = document.id, newPath = target))

            library.allFolders.forEach { folder ->
                assertEquals(expected = setOf(target.value), actual = library.folders.paths(folder = folder))
            }
            assertEquals(expected = target, actual = library.catalog.documents.getValue(key = document.id).path)
        }
    }

    @Test
    fun `a rename that only changes the case is allowed`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")

            val outcome = library.moveDocument(
                documentId = document.id,
                newPath = RelativePath.of(raw = "letters/A.pdf"),
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(expected = setOf("letters/A.pdf"), actual = library.folders.paths(folder = library.syncB))
        }
    }

    @Test
    fun `a name another file has, ignoring case, is refused`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")
            val other = library.add(path = "letters/b.pdf", content = "beta")

            val outcome = library.moveDocument(documentId = other.id, newPath = RelativePath.of(raw = "letters/A.PDF"))

            assertEquals(expected = OperationOutcome.NameTaken, actual = outcome)
            assertEquals(
                expected = "beta",
                actual = library.folders.content(folder = library.primary, path = "letters/b.pdf"),
            )
        }
    }

    @Test
    fun `a name the file name policy refuses is not used`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")

            val outcome = library.moveDocument(
                documentId = document.id,
                newPath = RelativePath.of(raw = "letters/a?.pdf"),
            )

            assertEquals(
                expected = OperationOutcome.InvalidName(violation = FileNameViolation.FORBIDDEN_CHARACTER),
                actual = outcome,
            )
        }
    }
}
