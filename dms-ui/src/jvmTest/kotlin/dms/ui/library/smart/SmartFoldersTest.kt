package dms.ui.library.smart

import dms.application.catalog.model.CatalogEdit
import dms.application.document.model.DetailEdit
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.dialog.DialogController
import dms.ui.library.smart.model.SmartFolderEntryModel
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SmartFoldersTest {

    private val dialogs = DialogController()
    private val selected = MutableStateFlow<SavedSearchIdModel?>(value = null)
    private val tax = TagNameModel.of(raw = "tax")
    private var current = DocumentQueryModel()

    @Test
    fun `what the library shows now is saved as a smart folder under the typed name`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)
            current = DocumentQueryModel(tags = TagFilterModel(all = setOf(tax)))

            smartFolders.saveCurrent()
            val title = dialogs.input.value?.title
            dialogs.submit(text = "  Tax papers ")
            app.idle()

            val saved = SampleLibrary.catalog(app = app).settings.savedSearches.single()
            assertEquals(expected = "Save as smart folder", actual = title)
            assertEquals(expected = "Tax papers", actual = saved.name)
            assertEquals(expected = current, actual = saved.query)
            assertEquals(expected = UiMessageModel(text = "Saved Tax papers"), actual = app.session.state.value.message)
        }
    }

    @Test
    fun `a blank name saves no smart folder`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)

            smartFolders.saveCurrent()
            dialogs.submit(text = "   ")
            app.idle()

            assertTrue(actual = SampleLibrary.catalog(app = app).settings.savedSearches.isEmpty())
        }
    }

    @Test
    fun `entries count their documents, leave text searches uncounted and mark a missing category`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf", "c.pdf")
            listOf("a.pdf", "b.pdf").forEach { path ->
                app.services.documents.editCatalog(
                    edit = CatalogEdit.AddTags(documentId = documents.getValue(key = path).id, tags = setOf(tax)),
                )
            }
            app.services.documents.editCatalog(edit = CatalogEdit.AddCategory(name = "Invoices", parentId = null))
            val invoices = SampleLibrary.catalog(app = app).categories.values.single()
            save(app = app, id = "tax", query = DocumentQueryModel(tags = TagFilterModel(all = setOf(tax))))
            save(app = app, id = "text", query = DocumentQueryModel(contentText = "bakery"))
            save(app = app, id = "invoices", query = DocumentQueryModel(category = invoices.id))
            app.services.documents.editCatalog(edit = CatalogEdit.RemoveCategory(categoryId = invoices.id))

            val entries = smartFoldersOf(app = app, scope = backgroundScope)
                .entriesOf(catalog = SampleLibrary.catalog(app = app))

            assertEquals(
                expected = listOf(
                    entry(id = "tax", count = 2, invalid = false),
                    entry(id = "text", count = null, invalid = false),
                    entry(id = "invoices", count = 0, invalid = true),
                ),
                actual = entries,
            )
        }
    }

    @Test
    fun `a rename keeps the query and replacing takes what the library shows now`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val id = save(app = app, id = "tax", query = DocumentQueryModel(tags = TagFilterModel(all = setOf(tax))))
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)
            current = DocumentQueryModel(untagged = true)

            smartFolders.rename(id = id)
            val initial = dialogs.input.value?.initialValue
            dialogs.submit(text = "Taxes")
            app.idle()
            val renamed = smartFolders.searchOf(id = id)
            smartFolders.replaceWithCurrent(id = id)
            app.idle()

            assertEquals(expected = "tax", actual = initial)
            assertEquals(expected = "Taxes", actual = renamed?.name)
            assertEquals(
                expected = DocumentQueryModel(tags = TagFilterModel(all = setOf(tax))),
                actual = renamed?.query,
            )
            assertEquals(
                expected = SavedSearchModel(id = id, name = "Taxes", query = current),
                actual = smartFolders.searchOf(id = id),
            )
        }
    }

    @Test
    fun `deleting asks first and removes only the saved search`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "a.pdf")
            val id = save(app = app, id = "all", query = DocumentQueryModel())
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)

            smartFolders.delete(id = id)
            dialogs.dismissConfirmation()
            app.idle()
            val kept = smartFolders.searchOf(id = id)
            smartFolders.delete(id = id)
            val title = dialogs.confirmation.value?.title
            dialogs.confirm()
            app.idle()

            assertEquals(expected = "all", actual = kept?.name)
            assertEquals(expected = "Delete smart folder \"all\"", actual = title)
            assertNull(actual = smartFolders.searchOf(id = id))
            assertEquals(expected = 1, actual = SampleLibrary.catalog(app = app).documents.size)
        }
    }

    @Test
    fun `the presets are added once, skip taken names and use the library's spelling of the tax tag`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            app.services.documents.editCatalog(
                edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(TagNameModel.of(raw = "TAX"))),
            )
            save(app = app, id = "own", query = DocumentQueryModel(untagged = true), name = "untagged")
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)

            repeat(times = 2) {
                smartFolders.addPresets()
                app.idle()
            }

            val year = app.services.library.clock.today().year
            val searches = SampleLibrary.catalog(app = app).settings.savedSearches
            val taxSearch = searches.single { it.name == "Tax $year" }
            assertEquals(
                expected = listOf("untagged", "Uncategorized", "Tax $year", "Due in 30 days"),
                actual = searches.map { it.name },
            )
            assertEquals(expected = listOf("TAX"), actual = taxSearch.query.tags.all.map { it.value })
        }
    }

    @Test
    fun `a selected smart folder that needs the documents' text gets its matches from the content search`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf")
            val bakery = documents.getValue(key = "a.pdf")
            app.services.documents.editDocumentDetails(
                edit = DetailEdit.SetNote(documentId = bakery.id, note = "Receipt of the Sample Bakery"),
            )
            val text = save(app = app, id = "text", query = DocumentQueryModel(contentText = "BAKERY"))
            val plain = save(app = app, id = "plain", query = DocumentQueryModel())
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)

            selected.value = text
            val matches = smartFolders.textMatches.first { it != null }
            selected.value = plain
            val afterPlain = smartFolders.textMatches.first { it == null }

            assertEquals(expected = setOf(bakery.id), actual = matches)
            assertNull(actual = afterPlain)
        }
    }

    @Test
    fun `without an open library saving reports that no library is open`() {
        UiTest.run { app ->
            val smartFolders = smartFoldersOf(app = app, scope = backgroundScope)

            smartFolders.saveCurrent()
            dialogs.submit(text = "Tax papers")
            app.idle()

            assertEquals(
                expected = UiMessageModel(text = "No library is open", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    private fun smartFoldersOf(app: TestApp, scope: CoroutineScope): SmartFolders {
        return SmartFolders(
            services = app.services,
            session = app.session,
            dialogs = dialogs,
            currentQuery = { current },
            selected = selected,
            scope = scope,
        )
    }

    private fun entry(id: String, count: Int?, invalid: Boolean): SmartFolderEntryModel {
        return SmartFolderEntryModel(id = SavedSearchIdModel(value = id), name = id, count = count, invalid = invalid)
    }

    private suspend fun save(
        app: TestApp,
        id: String,
        query: DocumentQueryModel,
        name: String = id,
    ): SavedSearchIdModel {
        val search = SavedSearchModel(id = SavedSearchIdModel(value = id), name = name, query = query)
        app.services.documents.changeSettings(subject = "Smart folder $name") { it.withSearch(search = search) }
        return search.id
    }
}
