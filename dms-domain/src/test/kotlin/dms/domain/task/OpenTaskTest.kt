package dms.domain.task

import dms.domain.TestData
import dms.domain.change.DetectedChange
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.task.model.TaskKeyModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class OpenTaskTest {

    private val path = RelativePathModel.of(raw = "inbox/scan.pdf")
    private val newFile = DetectedChange.NewFile(
        path = path,
        presentIn = setOf(LibraryFolderIdModel(value = "main")),
        size = 1,
        contentHash = TestData.hash(seed = 1),
    )

    @Test
    fun `every detected change maps to the task that resolves it`() {
        assertIs<OpenTask.AdoptNewFile>(value = OpenTask.forChange(change = newFile))
        val conflict = DetectedChange.ContentConflict(path = path, contentHashes = emptyMap())
        assertIs<OpenTask.ResolveContentConflict>(value = OpenTask.forChange(change = conflict))
    }

    @Test
    fun `the same change found twice yields the same key`() {
        val first = OpenTask.forChange(change = newFile)
        val second = OpenTask.forChange(change = newFile.copy(size = 2))

        assertEquals(expected = TaskKeyModel(value = "new:inbox/scan.pdf"), actual = first.key)
        assertEquals(expected = first.key, actual = second.key)
    }
}
