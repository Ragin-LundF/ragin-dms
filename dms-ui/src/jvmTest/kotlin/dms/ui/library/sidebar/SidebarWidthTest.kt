package dms.ui.library.sidebar

import androidx.compose.ui.unit.dp
import dms.ui.theme.Spacing
import kotlin.test.Test
import kotlin.test.assertEquals

class SidebarWidthTest {

    @Test
    fun `dragging widens and narrows the sidebar`() {
        val sidebar = SidebarWidth()

        sidebar.resize(delta = 40.dp)
        sidebar.resize(delta = (-10).dp)

        assertEquals(expected = Spacing.sidebarWidth + 30.dp, actual = sidebar.width.value)
    }

    @Test
    fun `the width stays between its limits`() {
        val sidebar = SidebarWidth()

        sidebar.resize(delta = 2000.dp)
        assertEquals(expected = Spacing.sidebarMaxWidth, actual = sidebar.width.value)

        sidebar.resize(delta = (-2000).dp)
        assertEquals(expected = Spacing.sidebarMinWidth, actual = sidebar.width.value)
    }
}
