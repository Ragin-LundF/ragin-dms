package dms.ui.library.picker

import dms.domain.library.model.RelativePathModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class FolderPickerTest {

    private val home = RelativePathModel.of(raw = "Home")
    private val energy = RelativePathModel.of(raw = "Home/Energy")
    private val letters = RelativePathModel.of(raw = "Letters")

    @Test
    fun `confirming hands the selected folder over and closes`() {
        val picked = mutableListOf<RelativePathModel>()
        val picker = FolderPicker()
        picker.open(title = "Move 2 documents") { picked += it }

        picker.confirm()
        assertTrue(actual = picked.isEmpty())
        picker.select(path = letters)
        picker.confirm()

        assertEquals(expected = listOf(letters), actual = picked)
        assertNull(actual = picker.state.value)
    }

    @Test
    fun `the excluded folder and everything below it cannot be picked`() {
        val picker = FolderPicker()
        picker.open(title = "Move Home", excluded = home) {}

        picker.select(path = energy)
        picker.select(path = home)
        assertNull(actual = picker.state.value?.selected)

        picker.select(path = RelativePathModel.ROOT)
        assertEquals(expected = RelativePathModel.ROOT, actual = picker.state.value?.selected)
    }

    @Test
    fun `toggling opens and closes a folder of the tree`() {
        val picker = FolderPicker()
        picker.open(title = "Move") {}

        picker.toggle(path = home)
        assertEquals(expected = setOf(home), actual = picker.state.value?.expanded)
        picker.toggle(path = home)
        assertEquals(expected = emptySet(), actual = picker.state.value?.expanded)
    }
}
