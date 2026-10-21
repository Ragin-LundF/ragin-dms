package dms.storage.text

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.document.DocumentId
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

class FileTextStoreTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val store = FileTextStore()
    private val documentId = DocumentId(value = "doc-1")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `stored text is read back and named by the document id`() {
        runTest {
            val text = "Page one\nStra\u00dfe, caf\u00e9, \u00fcber"

            val result = store.write(folder = folder, documentId = documentId, text = text)

            assertIs<StorageResult.Success<*>>(value = result)
            assertEquals(expected = text, actual = store.read(folder = folder, documentId = documentId))
            assertEquals(expected = text, actual = Files.readString(root.resolve(".dms-text/doc-1.txt")))
        }
    }

    @Test
    fun `missing text is not an error`() {
        runTest {
            assertNull(actual = store.read(folder = folder, documentId = documentId))
        }
    }

    @Test
    fun `an offline folder fails to write and has no text`() {
        runTest {
            val offline = TestFolders.folder(location = temp.resolve("offline"))

            val result = store.write(folder = offline, documentId = documentId, text = "text")

            assertEquals(
                expected = StorageFailure.FOLDER_UNREACHABLE,
                actual = StorageResults.failureOf(result = result),
            )
            assertNull(actual = store.read(folder = offline, documentId = documentId))
        }
    }
}
