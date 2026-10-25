package dms.ui.settings.library

import dms.application.catalog.model.CatalogEdit
import dms.application.document.model.DetailEdit
import dms.domain.field.FieldPresets
import dms.domain.field.FieldRole
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LibrarySettingsControllerTest {

    @Test
    fun `a field gets its key from the label and a label with a taken or no key is refused`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            controller.state.first { it.settings != null }

            controller.addField(label = " Contract number ", type = FieldType.TEXT, role = null)
            app.idle()
            controller.state.first { it.settings?.fieldDefinitions?.size == 1 }
            val refusals = listOf("contract  NUMBER", "!!!").map { label ->
                controller.addField(label = label, type = FieldType.NUMBER, role = null)
                app.idle()
                app.session.state.value.message
            }

            val definition = SampleLibrary.catalog(app = app).settings.fieldDefinitions.single()
            val refused = UiMessageModel(text = "Choose another name for the field", isError = true)
            assertEquals(expected = FieldKeyModel(value = "contract-number"), actual = definition.key)
            assertEquals(expected = "Contract number", actual = definition.label)
            assertEquals(expected = listOf(refused, refused), actual = refusals)
        }
    }

    @Test
    fun `only a date field keeps the role it was given`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            controller.state.first { it.settings != null }

            controller.addField(label = "Renewal", type = FieldType.DATE, role = FieldRole.CANCEL_BY)
            app.idle()
            controller.addField(label = "Reference", type = FieldType.TEXT, role = FieldRole.CANCEL_BY)
            app.idle()

            val roles = SampleLibrary.catalog(app = app).settings.fieldDefinitions.associate { it.label to it.role }
            assertEquals(expected = mapOf("Renewal" to FieldRole.CANCEL_BY, "Reference" to null), actual = roles)
        }
    }

    @Test
    fun `the standard fields are added beside a field that already uses one of their keys`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            controller.state.first { it.settings != null }
            controller.addField(label = "Amount", type = FieldType.NUMBER, role = null)
            app.idle()

            controller.addStandardFields()
            app.idle()

            val definitions = SampleLibrary.catalog(app = app).settings.fieldDefinitions
            val amount = definitions.single { it.key == FieldPresets.AMOUNT.key }
            val keys = definitions.map { it.key }.toSet()
            assertEquals(expected = FieldPresets.ALL.map { it.key }.toSet(), actual = keys)
            assertEquals(expected = FieldType.NUMBER, actual = amount.type)
            assertEquals(
                expected = UiMessageModel(text = "Added the standard fields"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `removing a field keeps the values documents have`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val controller = controllerOf(app = app, scope = backgroundScope)
            val value = FieldValue.Text(text = "R-17")
            val reference = FieldKeyModel(value = "reference")
            controller.state.first { it.settings != null }
            controller.addField(label = "Reference", type = FieldType.TEXT, role = null)
            app.idle()
            app.services.documents.editDocumentDetails(
                edit = DetailEdit.SetFields(documentId = document.id, fields = mapOf(reference to value)),
            )

            controller.removeField(key = reference)
            app.idle()

            assertNull(actual = SampleLibrary.catalog(app = app).settings.field(key = reference))
            val fields = SampleLibrary.document(app = app, id = document.id).fields
            assertEquals(expected = mapOf(reference to value), actual = fields)
        }
    }

    @Test
    fun `the default currency is saved in capitals, a bad code is refused and empty removes it`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)

            controller.saveCurrency(typed = " eur ")
            app.idle()
            val saved = SampleLibrary.catalog(app = app).settings.layout.defaultCurrency?.value
            controller.saveCurrency(typed = "euro")
            app.idle()
            val refused = app.session.state.value.message
            val afterRefusal = SampleLibrary.catalog(app = app).settings.layout.defaultCurrency?.value
            controller.saveCurrency(typed = "  ")
            app.idle()

            assertEquals(expected = "EUR", actual = saved)
            assertEquals(
                expected = UiMessageModel(text = "Write a three-letter currency code such as EUR", isError = true),
                actual = refused,
            )
            assertEquals(expected = "EUR", actual = afterRefusal)
            assertNull(actual = SampleLibrary.catalog(app = app).settings.layout.defaultCurrency)
        }
    }

    @Test
    fun `embedding metadata in PDFs is switched on and off`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)

            controller.setEmbedMetadata(enabled = true)
            app.idle()
            val on = SampleLibrary.catalog(app = app).settings.layout.embedMetadata
            controller.setEmbedMetadata(enabled = false)
            app.idle()

            assertTrue(actual = on)
            assertFalse(actual = SampleLibrary.catalog(app = app).settings.layout.embedMetadata)
            assertEquals(
                expected = UiMessageModel(text = "PDFs stay as they are"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `a rule is switched off and removed`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            app.services.organize.acceptRuleProposal(
                proposal = RuleProposalModel(
                    correspondent = "Sample Utility",
                    categoryId = null,
                    tags = setOf(TagNameModel.of(raw = "energy")),
                    documentCount = 3,
                ),
            )
            val rule = controller.state.first { it.settings?.rules?.isNotEmpty() == true }.settings?.rules?.single()
            val id = checkNotNull(value = rule).id

            controller.setRuleEnabled(id = id, enabled = false)
            app.idle()
            val switchedOff = SampleLibrary.catalog(app = app).settings.rules.single().enabled
            val message = app.session.state.value.message
            controller.removeRule(id = id)
            app.idle()

            assertFalse(actual = switchedOff)
            assertEquals(expected = UiMessageModel(text = "Switched Sample Utility off"), actual = message)
            assertTrue(actual = SampleLibrary.catalog(app = app).settings.rules.isEmpty())
        }
    }

    @Test
    fun `the inbox path must name valid folders inside the library`() {
        UiTest.run { app ->
            val controller = controllerOf(app = app, scope = backgroundScope)

            val paths = listOf("", " / ", "Mail/a:b", " Mail / Inbox ")
            val problems = paths.map { controller.inboxPathProblem(path = it) }

            assertEquals(
                expected = listOf(
                    "Name a folder inside the library, such as Inbox",
                    "Name a folder inside the library, such as Inbox",
                    "The name contains one of < > : \" / \\ | ? *",
                    null,
                ),
                actual = problems,
            )
        }
    }

    @Test
    fun `switching the inbox on creates its folder in every library folder`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)

            controller.saveInbox(
                enabled = true,
                path = " Mail / Inbox ",
                aiAnalysis = false,
                aiTargetFolder = false,
                autoAnalyze = false,
            )
            app.idle()

            val inbox = SampleLibrary.catalog(app = app).settings.layout.inbox
            assertTrue(actual = inbox.enabled)
            assertEquals(expected = "Mail/Inbox", actual = inbox.path.value)
            assertFalse(actual = inbox.aiAnalysis)
            assertTrue(actual = Files.isDirectory(app.primary.resolve("Mail/Inbox")))
            assertTrue(actual = Files.isDirectory(app.sync.resolve("Mail/Inbox")))
            assertEquals(
                expected = UiMessageModel(text = "The inbox is Mail/Inbox in every library folder"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `an invalid inbox path saves nothing`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            val before = SampleLibrary.catalog(app = app).settings.layout.inbox

            controller.saveInbox(
                enabled = true,
                path = "a:b",
                aiAnalysis = true,
                aiTargetFolder = true,
                autoAnalyze = true,
            )
            app.idle()

            assertEquals(expected = before, actual = SampleLibrary.catalog(app = app).settings.layout.inbox)
            assertFalse(actual = Files.exists(app.primary.resolve("a:b")))
        }
    }

    @Test
    fun `a category is named by its id when the catalog no longer has it`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            app.services.documents.editCatalog(edit = CatalogEdit.AddCategory(name = "Invoices", parentId = null))
            val invoices = SampleLibrary.catalog(app = app).categories.values.single()

            assertEquals(expected = "Invoices", actual = controller.categoryName(id = invoices.id))
            assertEquals(expected = "gone-1", actual = controller.categoryName(id = CategoryIdModel(value = "gone-1")))
        }
    }

    @Test
    fun `without an open library there are no settings and a change reports that no library is open`() {
        UiTest.run { app ->
            val controller = controllerOf(app = app, scope = backgroundScope)

            controller.addField(label = "Reference", type = FieldType.TEXT, role = null)
            val afterAddField = app.session.state.value.message
            controller.saveCurrency(typed = "EUR")
            app.idle()

            assertNull(actual = controller.state.value.settings)
            assertNull(actual = afterAddField)
            assertEquals(
                expected = UiMessageModel(text = "No library is open", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    private fun controllerOf(app: TestApp, scope: CoroutineScope): LibrarySettingsController {
        return LibrarySettingsController(services = app.services, session = app.session, scope = scope)
    }
}
