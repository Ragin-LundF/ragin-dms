package dms.ui.library

import dms.ui.library.model.YearFilterModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class YearFilterTest {

    @Test
    fun `the year chart's bars are read as filters`() {
        assertEquals(expected = YearFilterModel(year = 2024), actual = YearFilterModel.parse(text = "2024"))
        assertEquals(
            expected = YearFilterModel(year = 2017, andEarlier = true),
            actual = YearFilterModel.parse(text = "≤2017"),
        )
        assertNull(actual = YearFilterModel.parse(text = "soon"))
    }

    @Test
    fun `a filter with earlier years matches every year up to its own`() {
        val filter = YearFilterModel(year = 2017, andEarlier = true)

        assertTrue(actual = filter.matches(year = 2001) && filter.matches(year = 2017) && !filter.matches(year = 2018))
    }
}
