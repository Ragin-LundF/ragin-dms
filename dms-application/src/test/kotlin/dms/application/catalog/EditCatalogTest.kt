package dms.application.catalog

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.domain.catalog.CatalogRejection
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class EditCatalogTest {

    @Test
    fun `a document is categorized and tagged by hand and saved to every folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")

            library.editCatalog(edit = CatalogEdit.AddCategory(name = "Statements", parentId = null))
            val category = library.catalog.categories.values.single()
            library.editCatalog(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = category.id))
            library.editCatalog(
                edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(TagNameModel.of(raw = "Tax"))),
            )
            val outcome = library.editCatalog(
                edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(TagNameModel.of(raw = "tax"))),
            )

            val catalog = assertIs<OperationOutcome.Done>(value = outcome).catalog
            val edited = catalog.documents.getValue(key = document.id)
            assertEquals(expected = category.id, actual = edited.categoryId)
            assertEquals(expected = listOf("Tax"), actual = edited.tags.map { it.value })
            library.allFolders.forEach { folder ->
                assertEquals(expected = catalog, actual = library.folders.catalogs[folder.id])
            }
        }
    }

    @Test
    fun `a category in use cannot be removed`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            library.editCatalog(edit = CatalogEdit.AddCategory(name = "Statements", parentId = null))
            val category = library.catalog.categories.values.single()
            library.editCatalog(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = category.id))

            val outcome = library.editCatalog(edit = CatalogEdit.RemoveCategory(categoryId = category.id))

            assertEquals(
                expected = OperationOutcome.Rejected(reason = CatalogRejection.CATEGORY_IN_USE),
                actual = outcome,
            )
        }
    }
}
