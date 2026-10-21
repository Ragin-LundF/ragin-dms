package dms.storage.state

import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.PendingOperation
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FileOperationJournalTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val path: Path = temp.resolve("journal.yaml")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `every kind of operation survives a restart`() {
        runTest {
            val operations = listOf(
                FolderOperation.CopyFromPrimary(path = TestFolders.path(raw = "a.pdf"), replaceExisting = true),
                FolderOperation.CopyFromPrimary(path = TestFolders.path(raw = "caf\u00e9/b.pdf")),
                FolderOperation.CreateFolder(path = TestFolders.path(raw = "letters")),
                FolderOperation.Move(from = TestFolders.path(raw = "a.pdf"), to = TestFolders.path(raw = "x/a.pdf")),
                FolderOperation.MoveToTrash(path = TestFolders.path(raw = "old.pdf")),
                FolderOperation.CopyText(documentId = DocumentId(value = "doc-1")),
                FolderOperation.SaveCatalog,
            ).mapIndexed { index, operation -> pending(id = "op-$index", operation = operation) }
            val journal = FileOperationJournal(path = path)

            operations.forEach { journal.record(operation = it) }

            assertEquals(expected = operations, actual = FileOperationJournal(path = path).pending())
        }
    }

    @Test
    fun `a removed operation is gone`() {
        runTest {
            val journal = FileOperationJournal(path = path)
            val first = pending(id = "op-1", operation = FolderOperation.SaveCatalog)
            val folder = FolderOperation.CreateFolder(path = TestFolders.path(raw = "a"))
            val second = pending(id = "op-2", operation = folder)
            journal.record(operation = first)
            journal.record(operation = second)

            journal.remove(id = "op-1")

            assertEquals(expected = listOf(second), actual = FileOperationJournal(path = path).pending())
        }
    }

    @Test
    fun `an operation of a newer version is skipped`() {
        runTest {
            Files.writeString(
                path,
                """
                operations:
                - id: op-1
                  folderId: sync
                  recordedAt: 2024-05-01T08:00:00Z
                  type: FUTURE_OPERATION
                """.trimIndent(),
            )

            assertEquals(expected = emptyList(), actual = FileOperationJournal(path = path).pending())
        }
    }

    private fun pending(id: String, operation: FolderOperation): PendingOperation {
        return PendingOperation(
            id = id,
            folderId = LibraryFolderId(value = "sync"),
            operation = operation,
            recordedAt = Instant.parse("2024-05-01T08:00:00Z"),
        )
    }
}
