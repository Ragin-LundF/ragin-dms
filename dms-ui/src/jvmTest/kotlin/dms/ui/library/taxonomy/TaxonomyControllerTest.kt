package dms.ui.library.taxonomy

import dms.application.catalog.model.CatalogEdit
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.dialog.DialogController
import dms.ui.library.model.LibrarySelection
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class TaxonomyControllerTest {

    private val dialogs = DialogController()
    private val selections = mutableListOf<LibrarySelection>()

    @Test
    fun `a category is renamed, and a name a sibling already uses is refused`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val taxonomy = taxonomyOf(app = app)
            val invoices = addCategory(app = app, name = "Invoices")
            val receipts = addCategory(app = app, name = "Receipts")

            taxonomy.renameCategory(category = receipts)
            assertEquals(expected = "Receipts", actual = dialogs.input.value?.initialValue)
            dialogs.submit(text = "  Bills ")
            app.idle()
            taxonomy.renameCategory(category = invoices)
            dialogs.submit(text = "bills")
            app.idle()

            val names = SampleLibrary.catalog(app = app).categories.values.map { it.name }.toSet()
            assertEquals(expected = setOf("Invoices", "Bills"), actual = names)
            assertEquals(
                expected = UiMessageModel(text = "A category with this name already exists there", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `a blank category name adds no category and is reported`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val taxonomy = taxonomyOf(app = app)

            taxonomy.newCategory(parentId = null)
            dialogs.submit(text = "   ")
            app.idle()

            assertTrue(actual = SampleLibrary.catalog(app = app).categories.isEmpty())
            assertEquals(expected = true, actual = app.session.state.value.message?.isError)
        }
    }

    @Test
    fun `moving a category says where it went and a move below itself is refused`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val taxonomy = taxonomyOf(app = app)
            val invoices = addCategory(app = app, name = "Invoices")
            val receipts = addCategory(app = app, name = "Receipts")

            taxonomy.moveCategory(category = receipts, parentId = invoices.id)
            app.idle()
            val moved = app.session.state.value.message
            taxonomy.moveCategory(category = invoices, parentId = receipts.id)
            app.idle()
            val cycle = app.session.state.value.message
            taxonomy.moveCategory(category = receipts, parentId = null)
            app.idle()

            assertEquals(expected = UiMessageModel(text = "Moved Receipts to Invoices"), actual = moved)
            assertEquals(
                expected = UiMessageModel(text = "A category cannot be placed below itself", isError = true),
                actual = cycle,
            )
            assertEquals(
                expected = UiMessageModel(text = "Moved Receipts to the top level"),
                actual = app.session.state.value.message,
            )
            assertNull(actual = SampleLibrary.catalog(app = app).categories[receipts.id]?.parentId)
        }
    }

    @Test
    fun `deleting a category asks first, then removes it and shows all documents`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val taxonomy = taxonomyOf(app = app)
            val invoices = addCategory(app = app, name = "Invoices")

            taxonomy.removeCategory(category = invoices)
            dialogs.dismissConfirmation()
            app.idle()
            val kept = SampleLibrary.catalog(app = app).categories.keys
            taxonomy.removeCategory(category = invoices)
            assertEquals(expected = "Delete category \"Invoices\"", actual = dialogs.confirmation.value?.title)
            dialogs.confirm()
            app.idle()

            assertEquals(expected = setOf(invoices.id), actual = kept)
            assertTrue(actual = SampleLibrary.catalog(app = app).categories.isEmpty())
            assertEquals(
                expected = listOf<LibrarySelection>(LibrarySelection.Folder(path = RelativePathModel.ROOT)),
                actual = selections,
            )
        }
    }

    @Test
    fun `a category that documents still use is kept and the reason is shown`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "letters/a.pdf").getValue(key = "letters/a.pdf")
            val taxonomy = taxonomyOf(app = app)
            val invoices = addCategory(app = app, name = "Invoices")
            app.services.documents.editCatalog(
                edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = invoices.id),
            )

            taxonomy.removeCategory(category = invoices)
            dialogs.confirm()
            app.idle()

            assertNotNull(actual = SampleLibrary.catalog(app = app).categories[invoices.id])
            assertEquals(
                expected = UiMessageModel(text = "Documents still use this category", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `renaming a tag renames it on every document and selects the new tag`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf")
            documents.values.forEach { tag(app = app, documentId = it.id, tag = "tax") }
            val taxonomy = taxonomyOf(app = app)

            taxonomy.renameTag(tag = TagNameModel.of(raw = "tax"))
            dialogs.submit(text = " Taxes   2024 ")
            app.idle()

            val renamed = TagNameModel.of(raw = "Taxes 2024")
            val catalog = SampleLibrary.catalog(app = app)
            assertEquals(expected = listOf("Taxes 2024"), actual = catalog.tags.map { it.value })
            assertTrue(actual = catalog.documents.values.all { it.tags == setOf(renamed) })
            assertEquals(expected = listOf<LibrarySelection>(LibrarySelection.Tag(tag = renamed)), actual = selections)
        }
    }

    @Test
    fun `a blank tag name is refused and nothing changes`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            tag(app = app, documentId = document.id, tag = "tax")
            val taxonomy = taxonomyOf(app = app)

            taxonomy.renameTag(tag = TagNameModel.of(raw = "tax"))
            dialogs.submit(text = "  ")
            app.idle()

            val tags = SampleLibrary.catalog(app = app).tags.map { it.value }
            assertEquals(expected = listOf("tax"), actual = tags)
            assertEquals(
                expected = UiMessageModel(text = "Not a valid tag", isError = true),
                actual = app.session.state.value.message,
            )
            assertTrue(actual = selections.isEmpty())
        }
    }

    @Test
    fun `removing a tag names the tagged documents, then removes it from them`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf", "c.pdf")
            listOf("a.pdf", "b.pdf").forEach { path ->
                tag(app = app, documentId = documents.getValue(key = path).id, tag = "tax")
            }
            val taxonomy = taxonomyOf(app = app)

            taxonomy.removeTag(tag = TagNameModel.of(raw = "TAX"))
            val affected = dialogs.confirmation.value?.affected.orEmpty().toSet()
            dialogs.confirm()
            app.idle()

            val catalog = SampleLibrary.catalog(app = app)
            assertEquals(expected = setOf("a.pdf", "b.pdf"), actual = affected)
            assertTrue(actual = catalog.tags.isEmpty())
            assertTrue(actual = catalog.documents.values.all { it.tags.isEmpty() })
            assertEquals(
                expected = listOf<LibrarySelection>(LibrarySelection.Folder(path = RelativePathModel.ROOT)),
                actual = selections,
            )
        }
    }

    @Test
    fun `an edit reports success only when told what happened, and a refusal always`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val taxonomy = taxonomyOf(app = app)
            val tags = setOf(TagNameModel.of(raw = "tax"))

            taxonomy.edit(edit = CatalogEdit.AddTags(documentId = document.id, tags = tags))
            app.idle()
            val silent = app.session.state.value.message
            taxonomy.edit(edit = CatalogEdit.RemoveTags(documentId = document.id, tags = tags), done = "Untagged a.pdf")
            app.idle()
            val told = app.session.state.value.message
            taxonomy.edit(edit = CatalogEdit.AddTags(documentId = DocumentIdModel(value = "missing"), tags = tags))
            app.idle()

            assertNull(actual = silent)
            assertEquals(expected = UiMessageModel(text = "Untagged a.pdf"), actual = told)
            assertEquals(
                expected = UiMessageModel(text = "The document is no longer in the catalog", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `without an open library an edit reports that no library is open`() {
        UiTest.run { app ->
            val taxonomy = taxonomyOf(app = app)

            taxonomy.newCategory(parentId = null)
            dialogs.submit(text = "Invoices")
            app.idle()

            assertEquals(
                expected = UiMessageModel(text = "No library is open", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    private fun taxonomyOf(app: TestApp): TaxonomyController {
        return TaxonomyController(
            services = app.services,
            session = app.session,
            dialogs = dialogs,
            select = { selections += it },
        )
    }

    private suspend fun addCategory(app: TestApp, name: String): CategoryModel {
        app.services.documents.editCatalog(edit = CatalogEdit.AddCategory(name = name, parentId = null))
        return SampleLibrary.catalog(app = app).categories.values.single { it.name == name }
    }

    private suspend fun tag(app: TestApp, documentId: DocumentIdModel, tag: String) {
        app.services.documents.editCatalog(
            edit = CatalogEdit.AddTags(documentId = documentId, tags = setOf(TagNameModel.of(raw = tag))),
        )
    }
}
