package dms.storage.fs

import dms.application.port.storage.model.ExternalFileModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.FolderRole
import dms.domain.library.model.RelativePathModel
import dms.storage.support.RecordingTrash
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.FileTime
import java.time.Instant
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class FileSystemDocumentStorageTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val primaryRoot: Path = Files.createDirectories(temp.resolve("primary"))
    private val syncRoot: Path = Files.createDirectories(temp.resolve("sync"))
    private val primary = TestFolders.folder(location = primaryRoot)
    private val sync = TestFolders.folder(location = syncRoot, id = "sync", role = FolderRole.SYNC)
    private val trash = RecordingTrash(directory = temp.resolve("trash"))
    private val storage = FileSystemDocumentStorage(trash = trash)

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `an imported file is copied with its hash and modification time`() {
        runTest {
            val source = TestFolders.write(file = temp.resolve("outside/scan.pdf"), content = "alpha")
            val modified = FileTime.from(Instant.parse("2023-06-01T12:00:00Z"))
            Files.setLastModifiedTime(source, modified)

            val result = storage.importFile(
                folder = primary,
                source = ExternalFileModel(location = source.toString()),
                target = TestFolders.path(raw = "letters/scan.pdf"),
            )

            val facts = StorageResults.valueOf(result = result)
            assertEquals(expected = TestFolders.hashOf(content = "alpha"), actual = facts.contentHash)
            val copy = primaryRoot.resolve("letters/scan.pdf")
            assertEquals(expected = "alpha", actual = TestFolders.read(file = copy))
            assertEquals(expected = modified, actual = Files.getLastModifiedTime(copy))
            assertEquals(expected = setOf("scan.pdf"), actual = TestFolders.namesIn(directory = copy.parent))
        }
    }

    @Test
    fun `an import never takes a name that differs only in case`() {
        runTest {
            TestFolders.write(file = primaryRoot.resolve("Note.pdf"), content = "kept")
            val source = TestFolders.write(file = temp.resolve("outside/note.pdf"), content = "new")

            val result = storage.importFile(
                folder = primary,
                source = ExternalFileModel(location = source.toString()),
                target = TestFolders.path(raw = "note.pdf"),
            )

            assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = result))
            assertEquals(expected = "kept", actual = TestFolders.read(file = primaryRoot.resolve("Note.pdf")))
        }
    }

    @Test
    fun `an import of a missing source fails with not found`() {
        runTest {
            val result = storage.importFile(
                folder = primary,
                source = ExternalFileModel(location = temp.resolve("outside/missing.pdf").toString()),
                target = TestFolders.path(raw = "missing.pdf"),
            )

            assertEquals(expected = StorageFailure.NOT_FOUND, actual = StorageResults.failureOf(result = result))
        }
    }

    @Test
    fun `a copy replaces an existing file only when asked and only after trashing it`() {
        runTest {
            TestFolders.write(file = primaryRoot.resolve("a.pdf"), content = "new")
            TestFolders.write(file = syncRoot.resolve("a.pdf"), content = "old")
            val path = TestFolders.path(raw = "a.pdf")

            val refused = storage.copyFile(from = primary, to = sync, path = path, replaceExisting = false)
            val replaced = storage.copyFile(from = primary, to = sync, path = path, replaceExisting = true)

            assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = refused))
            assertIs<StorageResult.Success<*>>(value = replaced)
            assertEquals(expected = "new", actual = TestFolders.read(file = syncRoot.resolve("a.pdf")))
            assertEquals(expected = "old", actual = trash.contentOf(name = "a.pdf"))
        }
    }

    @Test
    fun `a copy keeps the existing file when the trash refuses it`() {
        runTest {
            val refusing = FileSystemDocumentStorage(trash = RecordingTrash(directory = temp, accepts = false))
            TestFolders.write(file = primaryRoot.resolve("a.pdf"), content = "new")
            TestFolders.write(file = syncRoot.resolve("a.pdf"), content = "old")

            val result = refusing.copyFile(
                from = primary,
                to = sync,
                path = TestFolders.path(raw = "a.pdf"),
                replaceExisting = true,
            )

            assertEquals(expected = StorageFailure.IO_ERROR, actual = StorageResults.failureOf(result = result))
            assertEquals(expected = "old", actual = TestFolders.read(file = syncRoot.resolve("a.pdf")))
        }
    }

    @Test
    fun `a file stored with a decomposed name is found by its composed path`() {
        runTest {
            TestFolders.write(file = primaryRoot.resolve("café/menu.pdf"), content = "alpha")

            val facts = storage.readFacts(folder = primary, path = TestFolders.path(raw = "café/menu.pdf"))
            val names = storage.listNames(folder = primary, path = RelativePathModel.ROOT)

            assertIs<StorageResult.Success<*>>(value = facts)
            assertEquals(expected = StorageResult.Success(value = listOf("café")), actual = names)
        }
    }

    @Test
    fun `listing a missing directory fails with not found`() {
        runTest {
            val result = storage.listNames(folder = primary, path = TestFolders.path(raw = "nowhere"))

            assertEquals(expected = StorageFailure.NOT_FOUND, actual = StorageResults.failureOf(result = result))
        }
    }

    @Test
    fun `a folder that is not reachable is reported as such`() {
        runTest {
            val offline = TestFolders.folder(location = temp.resolve("offline"))

            val result = storage.listNames(folder = offline, path = RelativePathModel.ROOT)

            assertEquals(
                expected = StorageFailure.FOLDER_UNREACHABLE,
                actual = StorageResults.failureOf(result = result),
            )
            assertTrue(actual = Files.notExists(temp.resolve("offline")))
        }
    }

    @Test
    fun `two files count as the same content only if every byte is the same`() {
        runTest {
            TestFolders.write(file = primaryRoot.resolve("a.pdf"), content = "alpha-1")
            TestFolders.write(file = primaryRoot.resolve("copy/a.pdf"), content = "alpha-1")
            TestFolders.write(file = primaryRoot.resolve("b.pdf"), content = "alpha-2")

            val copy = storage.sameContent(
                folder = primary,
                first = TestFolders.path(raw = "a.pdf"),
                second = TestFolders.path(raw = "copy/a.pdf"),
            )
            val sameSize = storage.sameContent(
                folder = primary,
                first = TestFolders.path(raw = "a.pdf"),
                second = TestFolders.path(raw = "b.pdf"),
            )

            assertEquals(expected = true, actual = StorageResults.valueOf(result = copy))
            assertEquals(expected = false, actual = StorageResults.valueOf(result = sameSize))
        }
    }

    @Test
    fun `a file is never a copy of itself`() {
        runTest {
            TestFolders.write(file = primaryRoot.resolve("a.pdf"), content = "alpha")
            val path = TestFolders.path(raw = "a.pdf")

            val result = storage.sameContent(folder = primary, first = path, second = path)

            assertEquals(expected = false, actual = StorageResults.valueOf(result = result))
        }
    }
}
