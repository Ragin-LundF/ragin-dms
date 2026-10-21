package dms.ui.library.drag

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.library.drag.model.DragItem
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
    private val dragged = DragItem.DraggedDocument(document = document)
    private val invoices = Category(id = CategoryId(value = "invoices"), name = "Invoices")
    private val archive = DropTarget.Folder(path = RelativePath.of(raw = "archive"))
    private val tax = DropTarget.Tag(tag = TagName.of(raw = "tax"))

    @Test
    fun `releasing over a target drops the document there`() {
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = draggingWith(drops = drops)

        drag.start(item = dragged, at = Offset(x = 5f, y = 5f))
        assertNull(actual = drag.hovered)
        drag.move(by = Offset(x = 100f, y = 0f))
        assertEquals(expected = archive, actual = drag.hovered)
        drag.finish()

        assertEquals(expected = listOf<Pair<DragItem, DropTarget>>(dragged to archive), actual = drops)
        assertNull(actual = drag.dragged)
    }

    @Test
    fun `releasing outside every target drops nothing`() {
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = draggingWith(drops = drops)

        drag.start(item = dragged, at = Offset(x = 5f, y = 5f))
        drag.finish()

        assertEquals(expected = emptyList<Pair<DragItem, DropTarget>>(), actual = drops)
    }

    @Test
    fun `an unregistered or cancelled target receives nothing`() {
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = draggingWith(drops = drops)
        drag.unregister(target = archive)

        drag.start(item = dragged, at = Offset(x = 105f, y = 5f))
        drag.finish()
        drag.start(item = dragged, at = Offset(x = 205f, y = 5f))
        assertEquals(expected = tax, actual = drag.hovered)
        drag.cancel()

        assertEquals(expected = emptyList<Pair<DragItem, DropTarget>>(), actual = drops)
        assertNull(actual = drag.hovered)
    }

    @Test
    fun `a category goes only into another category or to the top level`() {
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = draggingWith(drops = drops)
        val own = DropTarget.Category(categoryId = invoices.id)
        val other = DropTarget.Category(categoryId = CategoryId(value = "taxes"))
        drag.register(target = own, bounds = Rect(left = 0f, top = 100f, right = 100f, bottom = 150f))
        drag.register(target = other, bounds = Rect(left = 100f, top = 100f, right = 200f, bottom = 150f))
        drag.register(
            target = DropTarget.TopLevelCategory,
            bounds = Rect(left = 200f, top = 100f, right = 300f, bottom = 150f),
        )
        val category = DragItem.DraggedCategory(category = invoices)

        drag.start(item = category, at = Offset(x = 105f, y = 5f))
        assertNull(actual = drag.hovered)
        drag.move(by = Offset(x = -100f, y = 100f))
        assertNull(actual = drag.hovered)
        drag.move(by = Offset(x = 100f, y = 0f))
        assertEquals(expected = other, actual = drag.hovered)
        drag.move(by = Offset(x = 100f, y = 0f))
        assertEquals(expected = DropTarget.TopLevelCategory, actual = drag.hovered)
        drag.finish()

        assertEquals(
            expected = listOf<Pair<DragItem, DropTarget>>(category to DropTarget.TopLevelCategory),
            actual = drops,
        )
    }

    @Test
    fun `a document is not dropped on the categories heading`() {
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = draggingWith(drops = drops)
        drag.register(
            target = DropTarget.TopLevelCategory,
            bounds = Rect(left = 0f, top = 100f, right = 100f, bottom = 150f),
        )

        drag.start(item = dragged, at = Offset(x = 5f, y = 105f))
        assertNull(actual = drag.hovered)
        drag.finish()

        assertEquals(expected = emptyList<Pair<DragItem, DropTarget>>(), actual = drops)
    }

    private fun draggingWith(drops: MutableList<Pair<DragItem, DropTarget>>): DocumentDrag {
        val drag = DocumentDrag { dropped, target -> drops += dropped to target }
        drag.register(target = archive, bounds = Rect(left = 100f, top = 0f, right = 200f, bottom = 50f))
        drag.register(target = tax, bounds = Rect(left = 200f, top = 0f, right = 300f, bottom = 50f))
        return drag
    }
}
