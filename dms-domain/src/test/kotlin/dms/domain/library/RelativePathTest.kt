package dms.domain.library

import dms.domain.library.model.RelativePathModel
import java.text.Normalizer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RelativePathTest {

    @Test
    fun `decomposed and composed umlauts give the same path`() {
        val decomposed = Normalizer.normalize("Ärger/Übersicht.pdf", Normalizer.Form.NFD)
        val composed = Normalizer.normalize("Ärger/Übersicht.pdf", Normalizer.Form.NFC)

        assertEquals(expected = RelativePathModel.of(raw = composed), actual = RelativePathModel.of(raw = decomposed))
    }

    @Test
    fun `segments file name and parent describe the path`() {
        val path = RelativePathModel.of(raw = "projects/alpha/report.pdf")

        assertEquals(expected = listOf("projects", "alpha", "report.pdf"), actual = path.segments)
        assertEquals(expected = "report.pdf", actual = path.fileName)
        assertEquals(expected = RelativePathModel.of(raw = "projects/alpha"), actual = path.parent)
    }

    @Test
    fun `root has no segments and no parent`() {
        assertTrue(actual = RelativePathModel.ROOT.isRoot)
        assertEquals(expected = emptyList(), actual = RelativePathModel.ROOT.segments)
        assertNull(actual = RelativePathModel.ROOT.parent)
        assertEquals(expected = RelativePathModel.ROOT, actual = RelativePathModel.of(raw = "top.pdf").parent)
    }

    @Test
    fun `a trailing separator is ignored`() {
        assertEquals(
            expected = RelativePathModel.of(raw = "projects"),
            actual = RelativePathModel.of(raw = "projects/"),
        )
    }

    @Test
    fun `absolute paths and dot segments are rejected`() {
        listOf("/etc/passwd", "C:/data", "c:report", "a//b", "a/./b", "a/../b", "..").forEach { raw ->
            assertFailsWith<IllegalArgumentException>(message = raw) { RelativePathModel.of(raw = raw) }
        }
    }

    @Test
    fun `segments from the file system may contain backslashes`() {
        val path = RelativePathModel.fromSegments(segments = listOf("folder", "odd\\name.txt"))

        assertEquals(expected = "odd\\name.txt", actual = path.fileName)
    }

    @Test
    fun `isInside is strict and respects segment boundaries`() {
        val folder = RelativePathModel.of(raw = "projects")

        assertTrue(actual = RelativePathModel.of(raw = "projects/a.pdf").isInside(folder = folder))
        assertFalse(actual = RelativePathModel.of(raw = "projects-old/a.pdf").isInside(folder = folder))
        assertFalse(actual = folder.isInside(folder = folder))
        assertTrue(actual = folder.isInside(folder = RelativePathModel.ROOT))
    }

    @Test
    fun `relocate moves a path from one folder to another`() {
        val path = RelativePathModel.of(raw = "projects/alpha/report.pdf")

        val moved = path.relocate(
            from = RelativePathModel.of(raw = "projects"),
            to = RelativePathModel.of(raw = "archive/2024"),
        )

        assertEquals(expected = RelativePathModel.of(raw = "archive/2024/alpha/report.pdf"), actual = moved)
    }

    @Test
    fun `relocate refuses a path outside the source folder`() {
        assertFailsWith<IllegalArgumentException> {
            RelativePathModel.of(raw = "other/a.pdf").relocate(
                from = RelativePathModel.of(raw = "projects"),
                to = RelativePathModel.of(raw = "archive"),
            )
        }
    }

    @Test
    fun `parse accepts what of accepts and gives nothing for absolute or broken paths`() {
        val parsed = RelativePathModel.parse(raw = "Letters/2024/")

        assertEquals(expected = RelativePathModel.of(raw = "Letters/2024/"), actual = parsed)
        listOf("/abs", "C:/x", "a//b", "a/../b", "./a").forEach { raw ->
            assertNull(actual = RelativePathModel.parse(raw = raw), message = raw)
        }
    }
}
