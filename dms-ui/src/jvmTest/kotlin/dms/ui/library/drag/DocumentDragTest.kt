package dms.ui.library.drag

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.TagName
import dms.ui.library.drag.model.DropTarget
import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DocumentDragTest {

    private val document = Document(
        id = DocumentId(value = "a"),
        path = RelativePath.of(raw = "letters/a.pdf"),
        contentHash = ContentHash(value = "a".repeat(n = 64)),
        size = 1,
        addedAt = Instant.parse("2026-01-01T00:00:00Z"),
    )
    private val archive = DropTarget.Folder(path = RelativePath.of(raw = "archive"))
    private val tax = DropTarget.Tag(tag = TagName.of(raw = "tax"))

    @Test
    fun `releasing over a target drops the document there`() {
        val drops = mutableListOf<Pair<Document, DropTarget>>()
        val drag = draggingWith(drops = drops)

        drag.start(document = document, at = Offset(x = 5f, y = 5f))
        assertNull(actual = drag.hovered)
        drag.move(by = Offset(x = 100f, y = 0f))
        assertEquals(expected = archive, actual = drag.hovered)
        drag.finish()

        assertEquals(expected = listOf<Pair<Document, DropTarget>>(document to archive), actual = drops)
        assertNull(actual = drag.dragged)
    }

    @Test
    fun `releasing outside every target drops nothing`() {
        val drops = mutableListOf<Pair<Document, DropTarget>>()
        val drag = draggingWith(drops = drops)

        drag.start(document = document, at = Offset(x = 5f, y = 5f))
        drag.finish()

        assertEquals(expected = emptyList<Pair<Document, DropTarget>>(), actual = drops)
    }

    @Test
    fun `an unregistered or cancelled target receives nothing`() {
        val drops = mutableListOf<Pair<Document, DropTarget>>()
        val drag = draggingWith(drops = drops)
        drag.unregister(target = archive)

        drag.start(document = document, at = Offset(x = 105f, y = 5f))
        drag.finish()
        drag.start(document = document, at = Offset(x = 205f, y = 5f))
        assertEquals(expected = tax, actual = drag.hovered)
        drag.cancel()

        assertEquals(expected = emptyList<Pair<Document, DropTarget>>(), actual = drops)
        assertNull(actual = drag.hovered)
    }

    private fun draggingWith(drops: MutableList<Pair<Document, DropTarget>>): DocumentDrag {
        val drag = DocumentDrag { dropped, target -> drops += dropped to target }
        drag.register(target = archive, bounds = Rect(left = 100f, top = 0f, right = 200f, bottom = 50f))
        drag.register(target = tax, bounds = Rect(left = 200f, top = 0f, right = 300f, bottom = 50f))
        return drag
    }
}
