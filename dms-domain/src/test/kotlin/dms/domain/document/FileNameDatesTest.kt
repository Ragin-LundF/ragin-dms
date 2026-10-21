package dms.domain.document

import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class FileNameDatesTest {

    @Test
    fun `common date formats are recognized`() {
        mapOf(
            "2024-03-15 Sample bill.pdf" to "2024-03-15",
            "scan_2024_03_15.pdf" to "2024-03-15",
            "2024.03.15 note.pdf" to "2024-03-15",
            "IMG_20240315_101500.jpg" to "2024-03-15",
            "Sample 15.03.2024.pdf" to "2024-03-15",
            "15-03-2024 sample.pdf" to "2024-03-15",
            "2024-03 Sample.pdf" to "2024-03-01",
        ).forEach { (name, date) ->
            assertEquals(expected = LocalDate.parse(date), actual = FileNameDates.dateIn(name = name), message = name)
        }
    }

    @Test
    fun `the leftmost date wins`() {
        assertEquals(
            expected = LocalDate.parse("2023-12-01"),
            actual = FileNameDates.dateIn(name = "2023-12-01 reply to 15.11.2023.pdf"),
        )
    }

    @Test
    fun `numbers that are no date are ignored`() {
        listOf("Sample 12345678.pdf", "2024-13-40 Sample.pdf", "Order 0815.pdf", "1234-56.pdf", "Sample.pdf")
            .forEach { name -> assertNull(actual = FileNameDates.dateIn(name = name), message = name) }
    }
}
