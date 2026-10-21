package dms.storage.scan

import dms.application.port.storage.model.StorageFailure
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.LibraryId
import dms.domain.change.FileFacts
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.storage.support.RecordingListener
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.FileTime
import java.time.Instant
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs

class FileSystemLibraryScannerTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val scanner = FileSystemLibraryScanner(cacheDirectory = temp.resolve("cache"))
    private val empty = Catalog.empty(libraryId = LibraryId(value = "library-1"))

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `system clutter and the tool's own files are not listed`() {
        runTest {
            listOf(
                ".DS_Store",
                "._a.pdf",
                "Thumbs.db",
                "dms-catalog.yaml",
                "dms-catalog 2.yaml",
                ".dms-text/doc-1.txt",
                ".dms-trash/1/old.pdf",
                "letters/.dms-tmp-123",
                "letters/a.pdf",
            ).forEach { TestFolders.write(file = root.resolve(it), content = "x") }

            val files = scan(catalog = empty)

            assertEquals(expected = setOf(TestFolders.path(raw = "letters/a.pdf")), actual = files.keys)
        }
    }

    @Test
    fun `files the catalog does not know are hashed`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")

            val files = scan(catalog = empty)

            assertEquals(
                expected = FileFacts(size = 5, contentHash = TestFolders.hashOf(content = "alpha")),
                actual = files[TestFolders.path(raw = "a.pdf")],
            )
        }
    }

    @Test
    fun `a known file of the catalogued size is not hashed`() {
        runTest {
            // Same size, different content: only hashing would notice.
            TestFolders.write(file = root.resolve("a.pdf"), content = "ALPHA")

            val files = scan(catalog = catalogWith(path = "a.pdf", content = "alpha"))

            assertEquals(
                expected = FileFacts(size = 5, contentHash = null),
                actual = files[TestFolders.path(raw = "a.pdf")],
            )
        }
    }

    @Test
    fun `a known file with another size is hashed`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha, longer")

            val files = scan(catalog = catalogWith(path = "a.pdf", content = "alpha"))

            assertEquals(
                expected = TestFolders.hashOf(content = "alpha, longer"),
                actual = files[TestFolders.path(raw = "a.pdf")]?.contentHash,
            )
        }
    }

    @Test
    fun `a file changed since the last scan is hashed again`() {
        runTest {
            val file = TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")
            val catalog = catalogWith(path = "a.pdf", content = "alpha")
            scan(catalog = catalog)

            TestFolders.write(file = file, content = "ALPHA")
            Files.setLastModifiedTime(file, FileTime.from(Instant.parse("2030-01-01T00:00:00Z")))
            val files = scan(catalog = catalog)

            assertEquals(
                expected = TestFolders.hashOf(content = "ALPHA"),
                actual = files[TestFolders.path(raw = "a.pdf")]?.contentHash,
            )
        }
    }

    @Test
    fun `a cloud placeholder is listed under its real name unless the file is downloaded`() {
        runTest {
            TestFolders.write(file = root.resolve(".remote.pdf.icloud"), content = "stub")
            TestFolders.write(file = root.resolve(".local.pdf.icloud"), content = "stub")
            TestFolders.write(file = root.resolve("local.pdf"), content = "beta")

            val files = scan(catalog = catalogWith(path = "remote.pdf", content = "alpha"))

            assertEquals(
                expected = FileFacts(size = 5, contentHash = null),
                actual = files[TestFolders.path(raw = "remote.pdf")],
            )
            assertEquals(
                expected = TestFolders.hashOf(content = "beta"),
                actual = files[TestFolders.path(raw = "local.pdf")]?.contentHash,
            )
            assertEquals(expected = 2, actual = files.size)
        }
    }

    @Test
    fun `a decomposed file name is listed with its composed path`() {
        runTest {
            TestFolders.write(file = root.resolve("cafe\u0301/menu.pdf"), content = "alpha")

            val files = scan(catalog = empty)

            assertEquals(expected = setOf(TestFolders.path(raw = "caf\u00e9/menu.pdf")), actual = files.keys)
        }
    }

    @Test
    fun `the listener learns the total first and then every file`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")
            TestFolders.write(file = root.resolve("letters/b.pdf"), content = "beta")
            val first = RecordingListener()
            val second = RecordingListener()

            scanner.scan(folder = folder, catalog = empty, listener = first)
            scanner.scan(folder = folder, catalog = empty, listener = second)

            assertEquals(expected = listOf(2), actual = first.totals)
            assertEquals(expected = listOf("1 a.pdf true", "2 letters/b.pdf true"), actual = first.files)
            assertEquals(expected = listOf("1 a.pdf false", "2 letters/b.pdf false"), actual = second.files)
        }
    }

    @Test
    fun `a cancelled scan keeps the hashes it already computed`() {
        runTest {
            TestFolders.write(file = root.resolve("a.pdf"), content = "alpha")
            TestFolders.write(file = root.resolve("b.pdf"), content = "beta")
            val checkpointing = FileSystemLibraryScanner(cacheDirectory = temp.resolve("cache"), checkpointEvery = 1)
            val cancelling = RecordingListener(onFile = { throw CancellationException("stopped") })
            val after = RecordingListener()

            assertFailsWith<CancellationException> {
                checkpointing.scan(folder = folder, catalog = empty, listener = cancelling)
            }
            checkpointing.scan(folder = folder, catalog = empty, listener = after)

            assertEquals(expected = listOf("1 a.pdf false", "2 b.pdf true"), actual = after.files)
        }
    }

    @Test
    fun `a folder that is not reachable is reported as such`() {
        runTest {
            val offline = TestFolders.folder(location = temp.resolve("offline"))

            val result = scanner.scan(folder = offline, catalog = empty)

            assertEquals(
                expected = StorageFailure.FOLDER_UNREACHABLE,
                actual = StorageResults.failureOf(result = result),
            )
        }
    }

    private suspend fun scan(catalog: Catalog): Map<RelativePath, FileFacts> {
        return StorageResults.valueOf(result = scanner.scan(folder = folder, catalog = catalog)).files
    }

    private fun catalogWith(path: String, content: String): Catalog {
        val document = Document(
            id = DocumentId(value = "doc-1"),
            path = TestFolders.path(raw = path),
            contentHash = TestFolders.hashOf(content = content),
            size = content.length.toLong(),
            addedAt = Instant.parse("2024-01-01T10:00:00Z"),
        )
        return assertIs<CatalogUpdate.Applied>(value = empty.addDocument(document = document)).catalog
    }
}
