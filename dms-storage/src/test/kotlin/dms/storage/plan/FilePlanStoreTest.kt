package dms.storage.plan

import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.storage.model.StorageFailure
import dms.domain.document.model.DocumentIdModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class FilePlanStoreTest {

    private val temp: Path = Files.createTempDirectory("dms-plans-")
    private val root: Path = Files.createDirectories(temp.resolve("library"))
    private val folder = TestFolders.folder(location = root)
    private val store = FilePlanStore()

    private fun record(name: String): RestructureRecordModel {
        return RestructureRecordModel(
            name = name,
            appliedAt = Instant.parse("2026-10-22T08:00:00Z"),
            template = "{category}/{year}",
            moves = listOf(
                PlannedMoveModel(
                    documentId = DocumentIdModel(value = "doc-1"),
                    from = TestFolders.path(raw = "loose/Grüße.pdf"),
                    to = TestFolders.path(raw = "Letters/2024/Grüße.pdf"),
                ),
            ),
        )
    }

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a record is written to the hidden plans folder and read back`() {
        runTest {
            StorageResults.valueOf(result = store.write(folder = folder, record = record(name = "20261022-080000")))

            assertEquals(
                expected = record(name = "20261022-080000"),
                actual = store.read(folder = folder, name = "20261022-080000"),
            )
            assertEquals(expected = true, actual = Files.isRegularFile(root.resolve(".dms-plans/20261022-080000.yaml")))
        }
    }

    @Test
    fun `names are listed newest first and a missing or broken record reads as none`() {
        runTest {
            store.write(folder = folder, record = record(name = "20260101-000000"))
            store.write(folder = folder, record = record(name = "20261022-080000"))
            TestFolders.write(file = root.resolve(".dms-plans/broken.yaml"), content = "moves: [")

            assertEquals(
                expected = listOf("broken", "20261022-080000", "20260101-000000"),
                actual = store.names(folder = folder),
            )
            assertNull(actual = store.read(folder = folder, name = "broken"))
            assertNull(actual = store.read(folder = folder, name = "missing"))
        }
    }

    @Test
    fun `an offline folder fails to write and has no records`() {
        runTest {
            val offline = TestFolders.folder(location = temp.resolve("offline"))

            assertEquals(
                expected = StorageFailure.FOLDER_UNREACHABLE,
                actual = StorageResults.failureOf(result = store.write(folder = offline, record = record(name = "x"))),
            )
            assertEquals(expected = emptyList(), actual = store.names(folder = offline))
        }
    }
}
