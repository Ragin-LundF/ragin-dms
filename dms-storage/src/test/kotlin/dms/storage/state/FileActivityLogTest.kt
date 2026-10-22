package dms.storage.state

import dms.application.port.state.model.ActivityEntryModel
import dms.application.port.state.model.ActivityKind
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FileActivityLogTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val path: Path = temp.resolve("activity.yaml")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `entries come back newest first and only up to the capacity`() {
        runTest {
            val log = FileActivityLog(path = path, capacity = 3)
            val entries = (1..4).map { entry(minute = it, subject = "file-$it.pdf") }

            entries.forEach { log.record(entry = it) }

            assertEquals(
                expected = entries.reversed().take(n = 3),
                actual = FileActivityLog(path = path).recent(limit = 10),
            )
            assertEquals(expected = listOf(entries.last()), actual = log.recent(limit = 1))
        }
    }

    @Test
    fun `entries of an unknown kind are skipped`() {
        runTest {
            Files.writeString(
                path,
                """
                entries:
                - at: 2024-05-01T08:00:00Z
                  kind: FUTURE_KIND
                  subject: a.pdf
                - at: 2024-05-01T07:00:00Z
                  kind: DOCUMENT_ADDED
                  subject: b.pdf
                """.trimIndent(),
            )

            assertEquals(
                expected = listOf(
                    ActivityEntryModel(
                        at = Instant.parse("2024-05-01T07:00:00Z"),
                        kind = ActivityKind.DOCUMENT_ADDED,
                        subject = "b.pdf",
                    ),
                ),
                actual = FileActivityLog(path = path).recent(limit = 10),
            )
        }
    }

    private fun entry(minute: Int, subject: String): ActivityEntryModel {
        return ActivityEntryModel(
            at = Instant.parse("2024-05-01T08:00:00Z").plusSeconds(minute * 60L),
            kind = ActivityKind.DOCUMENT_ADDED,
            subject = subject,
        )
    }
}
