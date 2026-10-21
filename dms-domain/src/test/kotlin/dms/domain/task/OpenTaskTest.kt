package dms.domain.task

import dms.domain.TestData
import dms.domain.change.DetectedChange
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class OpenTaskTest {

    private val path = RelativePath.of(raw = "inbox/scan.pdf")
    private val newFile = DetectedChange.NewFile(
        path = path,
        presentIn = setOf(LibraryFolderId(value = "main")),
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

        assertEquals(expected = TaskKey(value = "new:inbox/scan.pdf"), actual = first.key)
        assertEquals(expected = first.key, actual = second.key)
    }
}
