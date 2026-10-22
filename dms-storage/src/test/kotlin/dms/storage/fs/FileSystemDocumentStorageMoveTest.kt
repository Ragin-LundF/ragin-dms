package dms.storage.fs

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.RelativePathModel
import dms.storage.support.RecordingTrash
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class FileSystemDocumentStorageMoveTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val trash = RecordingTrash(directory = temp.resolve("trash"))
    private val storage = FileSystemDocumentStorage(trash = trash)

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a move creates missing parent folders`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")

            val result = storage.move(
                folder = folder,
                from = TestFolders.path(raw = "a.pdf"),
                to = TestFolders.path(raw = "archive/2024/a.pdf"),
            )

            assertIs<StorageResult.Success<*>>(value = result)
            assertEquals(expected = "alpha", actual = TestFolders.read(file = root.resolve("archive/2024/a.pdf")))
            assertEquals(expected = setOf("archive"), actual = TestFolders.namesIn(directory = root))
        }
    }

    @Test
    fun `a rename that only changes the case works`() {
        runTest {
            TestFolders.write(file = root.resolve("note.pdf"), content = "alpha")

            val result = storage.move(
                folder = folder,
                from = TestFolders.path(raw = "note.pdf"),
                to = TestFolders.path(raw = "Note.pdf"),
            )

            assertIs<StorageResult.Success<*>>(value = result)
            assertEquals(expected = setOf("Note.pdf"), actual = TestFolders.namesIn(directory = root))
            assertEquals(expected = "alpha", actual = TestFolders.read(file = root.resolve("Note.pdf")))
        }
    }

    @Test
    fun `a move onto a name that differs only in case fails`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")
            TestFolders.write(file = root.resolve("b.pdf"), content = "beta")

            val result = storage.move(
                folder = folder,
                from = TestFolders.path(raw = "a.pdf"),
                to = TestFolders.path(raw = "B.pdf"),
            )

            assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = result))
            assertEquals(expected = "beta", actual = TestFolders.read(file = root.resolve("b.pdf")))
        }
    }

    @Test
    fun `a move of a missing entry fails with not found`() {
        runTest {
            val result = storage.move(
                folder = folder,
                from = TestFolders.path(raw = "missing.pdf"),
                to = TestFolders.path(raw = "other.pdf"),
            )

            assertEquals(expected = StorageFailure.NOT_FOUND, actual = StorageResults.failureOf(result = result))
        }
    }

    @Test
    fun `a trashed file goes to the system trash`() {
        runTest {
            TestFolders.write(file = root.resolve("letters/a.pdf"), content = "alpha")

            val result = storage.moveToTrash(folder = folder, path = TestFolders.path(raw = "letters/a.pdf"))

            assertIs<StorageResult.Success<*>>(value = result)
            assertEquals(expected = "alpha", actual = trash.contentOf(name = "a.pdf"))
            assertEquals(expected = emptySet(), actual = TestFolders.namesIn(directory = root.resolve("letters")))
        }
    }

    @Test
    fun `creating an existing folder succeeds but a clash with another entry does not`() {
        runTest {
            TestFolders.write(file = root.resolve("notes"), content = "a file")

            val first = storage.createFolder(folder = folder, path = TestFolders.path(raw = "letters"))
            val again = storage.createFolder(folder = folder, path = TestFolders.path(raw = "letters"))
            val otherCase = storage.createFolder(folder = folder, path = TestFolders.path(raw = "Letters"))
            val onFile = storage.createFolder(folder = folder, path = TestFolders.path(raw = "notes"))

            assertIs<StorageResult.Success<*>>(value = first)
            assertIs<StorageResult.Success<*>>(value = again)
            assertEquals(
                expected = StorageFailure.ALREADY_EXISTS,
                actual = StorageResults.failureOf(result = otherCase),
            )
            assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = onFile))
        }
    }

    @Test
    fun `the facts of a file are always hashed`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")

            val result = storage.readFacts(folder = folder, path = TestFolders.path(raw = "a.pdf"))

            val facts = StorageResults.valueOf(result = result)
            assertEquals(expected = 5L, actual = facts.size)
            assertEquals(expected = TestFolders.hashOf(content = "alpha"), actual = facts.contentHash)
            val onFolder = storage.readFacts(folder = folder, path = RelativePathModel.ROOT)
            assertEquals(expected = StorageFailure.NOT_FOUND, actual = StorageResults.failureOf(result = onFolder))
        }
    }
}
