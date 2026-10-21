package dms.ui.library

import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals

class LibraryTreesTest {

    @Test
    fun `folders follow their parents and count every document below them`() {
        val documents = listOf(
            document(id = "1", path = "letters/2024/a.pdf"),
            document(id = "2", path = "letters/b.pdf"),
            document(id = "3", path = "Archive/c.pdf"),
            document(id = "4", path = "top.pdf"),
        )

        val entries = LibraryTrees.folders(
            documents = documents,
            createdFolders = setOf(RelativePath.of(raw = "letters/empty")),
        )

        assertEquals(
            expected = listOf(
                "" to 4,
                "Archive" to 1,
                "letters" to 2,
                "letters/2024" to 1,
                "letters/empty" to 0,
            ),
            actual = entries.map { it.path.value to it.documentCount },
        )
        assertEquals(expected = listOf(0, 1, 1, 2, 2), actual = entries.map { it.depth })
    }

    @Test
    fun `subfolders show only below an unfolded folder`() {
        val documents = listOf(
            document(id = "1", path = "letters/2024/march/a.pdf"),
            document(id = "2", path = "Archive/old/c.pdf"),
        )

        val folded = LibraryTrees.folders(documents = documents, createdFolders = emptySet(), expanded = emptySet())
        val unfolded = LibraryTrees.folders(
            documents = documents,
            createdFolders = emptySet(),
            expanded = setOf(RelativePath.of(raw = "letters")),
        )

        assertEquals(expected = listOf("", "Archive", "letters"), actual = folded.map { it.path.value })
        assertEquals(expected = listOf(true, true, true), actual = folded.map { it.hasChildren })
        assertEquals(
            expected = listOf("", "Archive", "letters", "letters/2024"),
            actual = unfolded.map { it.path.value },
        )
    }

    @Test
    fun `subcategories show only below an unfolded category`() {
        val parent = Category(id = CategoryId(value = "p"), name = "Invoices")
        val child = Category(id = CategoryId(value = "c"), name = "Energy", parentId = parent.id)

        val folded = LibraryTrees.categories(
            categories = listOf(parent, child),
            documents = emptyList(),
            expanded = emptySet(),
        )

        assertEquals(expected = listOf("Invoices"), actual = folded.map { it.category.name })
        assertEquals(expected = true, actual = folded.single().hasChildren)
    }

    @Test
    fun `subcategories follow their parent, sorted by name`() {
        val parent = Category(id = CategoryId(value = "p"), name = "Invoices")
        val child = Category(id = CategoryId(value = "c"), name = "Energy", parentId = parent.id)
        val other = Category(id = CategoryId(value = "o"), name = "Contracts")
        val documents = listOf(document(id = "1", path = "a.pdf").copy(categoryId = child.id))

        val entries = LibraryTrees.categories(categories = listOf(parent, child, other), documents = documents)

        assertEquals(
            expected = listOf(Triple("Contracts", 0, 0), Triple("Invoices", 0, 0), Triple("Energy", 1, 1)),
            actual = entries.map { Triple(it.category.name, it.depth, it.documentCount) },
        )
    }

    private fun document(id: String, path: String): Document {
        return Document(
            id = DocumentId(value = id),
            path = RelativePath.of(raw = path),
            contentHash = ContentHash(value = "0".repeat(n = 64)),
            size = 1,
            addedAt = Instant.parse("2024-01-01T10:00:00Z"),
        )
    }
}
