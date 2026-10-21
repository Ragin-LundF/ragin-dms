package dms.ui.library

import dms.ui.library.model.YearFilter
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class YearFilterTest {

    @Test
    fun `the year chart's bars are read as filters`() {
        assertEquals(expected = YearFilter(year = 2024), actual = YearFilter.parse(text = "2024"))
        assertEquals(expected = YearFilter(year = 2017, andEarlier = true), actual = YearFilter.parse(text = "≤2017"))
        assertNull(actual = YearFilter.parse(text = "soon"))
    }

    @Test
    fun `a filter with earlier years matches every year up to its own`() {
        val filter = YearFilter(year = 2017, andEarlier = true)

        assertTrue(actual = filter.matches(year = 2001) && filter.matches(year = 2017) && !filter.matches(year = 2018))
    }
}
