package dms.storage.fs

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.storage.support.StorageResults
import dms.storage.support.TestFolders
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class AtomicFilesTest {

    private val temp: Path = Files.createTempDirectory("dms-atomic-")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `a file that appears after the copy started is not replaced`() {
        val target = TestFolders.write(file = temp.resolve("letter.pdf"), content = "arrived meanwhile")
        val copied = TestFolders.write(file = AtomicFiles.tempFileFor(target = target), content = "copied")

        val result = AtomicFiles.moveWithoutReplacing(temp = copied, target = target)

        assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = result))
        assertEquals(expected = "arrived meanwhile", actual = TestFolders.read(file = target))
    }

    @Test
    fun `a copy is placed under its name and leaves no temporary file`() {
        val source = TestFolders.write(file = temp.resolve("source/letter.pdf"), content = "letter")
        val target = temp.resolve("target/letter.pdf")

        val result = AtomicFiles.copy(source = source, target = target)

        assertIs<StorageResult.Success<*>>(value = result)
        assertEquals(expected = "letter", actual = TestFolders.read(file = target))
        assertEquals(expected = setOf("letter.pdf"), actual = TestFolders.namesIn(directory = target.parent))
    }

    @Test
    fun `a copy onto a name that differs only by case fails and keeps the existing file`() {
        val source = TestFolders.write(file = temp.resolve("source/letter.pdf"), content = "new")
        val existing = TestFolders.write(file = temp.resolve("target/LETTER.pdf"), content = "existing")

        val result = AtomicFiles.copy(source = source, target = temp.resolve("target/letter.pdf"))

        assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = StorageResults.failureOf(result = result))
        assertEquals(expected = "existing", actual = TestFolders.read(file = existing))
        assertEquals(expected = setOf("LETTER.pdf"), actual = TestFolders.namesIn(directory = existing.parent))
    }

    @Test
    fun `write replaces the existing file`() {
        val target = TestFolders.write(file = temp.resolve("state.yaml"), content = "old")

        AtomicFiles.write(target = target, bytes = "new".toByteArray())

        assertEquals(expected = "new", actual = TestFolders.read(file = target))
        assertEquals(expected = setOf("state.yaml"), actual = TestFolders.namesIn(directory = temp))
    }
}
