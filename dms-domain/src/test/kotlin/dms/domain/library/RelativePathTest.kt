package dms.domain.library

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

        assertEquals(expected = RelativePath.of(raw = composed), actual = RelativePath.of(raw = decomposed))
    }

    @Test
    fun `segments file name and parent describe the path`() {
        val path = RelativePath.of(raw = "projects/alpha/report.pdf")

        assertEquals(expected = listOf("projects", "alpha", "report.pdf"), actual = path.segments)
        assertEquals(expected = "report.pdf", actual = path.fileName)
        assertEquals(expected = RelativePath.of(raw = "projects/alpha"), actual = path.parent)
    }

    @Test
    fun `root has no segments and no parent`() {
        assertTrue(actual = RelativePath.ROOT.isRoot)
        assertEquals(expected = emptyList(), actual = RelativePath.ROOT.segments)
        assertNull(actual = RelativePath.ROOT.parent)
        assertEquals(expected = RelativePath.ROOT, actual = RelativePath.of(raw = "top.pdf").parent)
    }

    @Test
    fun `a trailing separator is ignored`() {
        assertEquals(expected = RelativePath.of(raw = "projects"), actual = RelativePath.of(raw = "projects/"))
    }

    @Test
    fun `absolute paths and dot segments are rejected`() {
        listOf("/etc/passwd", "C:/data", "c:report", "a//b", "a/./b", "a/../b", "..").forEach { raw ->
            assertFailsWith<IllegalArgumentException>(message = raw) { RelativePath.of(raw = raw) }
        }
    }

    @Test
    fun `segments from the file system may contain backslashes`() {
        val path = RelativePath.fromSegments(segments = listOf("folder", "odd\\name.txt"))

        assertEquals(expected = "odd\\name.txt", actual = path.fileName)
    }

    @Test
    fun `isInside is strict and respects segment boundaries`() {
        val folder = RelativePath.of(raw = "projects")

        assertTrue(actual = RelativePath.of(raw = "projects/a.pdf").isInside(folder = folder))
        assertFalse(actual = RelativePath.of(raw = "projects-old/a.pdf").isInside(folder = folder))
        assertFalse(actual = folder.isInside(folder = folder))
        assertTrue(actual = folder.isInside(folder = RelativePath.ROOT))
    }

    @Test
    fun `relocate moves a path from one folder to another`() {
        val path = RelativePath.of(raw = "projects/alpha/report.pdf")

        val moved = path.relocate(from = RelativePath.of(raw = "projects"), to = RelativePath.of(raw = "archive/2024"))

        assertEquals(expected = RelativePath.of(raw = "archive/2024/alpha/report.pdf"), actual = moved)
    }

    @Test
    fun `relocate refuses a path outside the source folder`() {
        assertFailsWith<IllegalArgumentException> {
            RelativePath.of(raw = "other/a.pdf").relocate(
                from = RelativePath.of(raw = "projects"),
                to = RelativePath.of(raw = "archive"),
            )
        }
    }
}
