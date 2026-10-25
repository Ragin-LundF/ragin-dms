package dms.ui.session

import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DateInputTest {

    @Test
    fun `an ISO date with spaces around it is read`() {
        assertEquals(expected = LocalDate.of(2030, 12, 31), actual = DateInput.parse(raw = " 2030-12-31 "))
    }

    @Test
    fun `anything else is no date`() {
        listOf("31.12.2030", "2030-02-30", "", "soon").forEach { raw ->
            assertNull(actual = DateInput.parse(raw = raw), message = raw)
        }
    }
}
