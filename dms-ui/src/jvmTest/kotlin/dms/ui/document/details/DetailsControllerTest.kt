package dms.ui.document.details

import dms.application.catalog.model.CatalogEdit
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.details.model.LinkCandidateModel
import dms.ui.session.DateInput
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import java.time.Instant
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DetailsControllerTest {

    private val amount = FieldPresets.AMOUNT.key
    private val due = FieldPresets.DUE_DATE.key

    @Test
    fun `typed field values are saved by their type and an emptied field loses its value`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)
            details.addStandardFields()
            app.idle()

            details.saveFields(document = document, typed = mapOf(amount to "12.50 eur", due to " 2026-12-31 "))
            app.idle()
            val saved = formattedFields(app = app, id = document.id)
            details.saveFields(
                document = SampleLibrary.document(app = app, id = document.id),
                typed = mapOf(amount to " ", due to "2026-12-31"),
            )
            app.idle()

            assertEquals(expected = mapOf(amount to "12.50 EUR", due to "2026-12-31"), actual = saved)
            assertEquals(expected = mapOf(due to "2026-12-31"), actual = formattedFields(app = app, id = document.id))
        }
    }

    @Test
    fun `a value that does not fit its field saves nothing and names the field`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)
            details.addStandardFields()
            app.idle()

            details.saveFields(document = document, typed = mapOf(amount to "12.50 EUR", due to "2026-02-30"))
            app.idle()

            assertTrue(actual = SampleLibrary.document(app = app, id = document.id).fields.isEmpty())
            assertEquals(
                expected = UiMessageModel(text = "\"Due date\": write a date such as 2026-12-31", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `a value of a field the library no longer defines is kept when the other fields are saved`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)
            details.addStandardFields()
            app.idle()
            details.saveFields(document = document, typed = mapOf(amount to "12.50 EUR"))
            app.idle()
            app.services.documents.changeSettings(subject = "Remove amount") { it.withoutField(key = amount) }

            details.saveFields(
                document = SampleLibrary.document(app = app, id = document.id),
                typed = mapOf(due to "2026-12-31"),
            )
            app.idle()

            assertNotEquals(illegal = true, actual = app.session.state.value.message?.isError)
            assertEquals(
                expected = mapOf(amount to "12.50 EUR", due to "2026-12-31"),
                actual = formattedFields(app = app, id = document.id),
            )
        }
    }

    @Test
    fun `the note is saved, the same text writes nothing and a blank note removes it`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)

            details.saveNote(document = document, note = "Paid by transfer  ")
            app.idle()
            val noted = SampleLibrary.document(app = app, id = document.id)
            val revision = SampleLibrary.catalog(app = app).revision
            details.saveNote(document = noted, note = "Paid by transfer ")
            app.idle()
            val unchanged = SampleLibrary.catalog(app = app).revision
            details.saveNote(document = noted, note = "   ")
            app.idle()

            assertEquals(expected = "Paid by transfer", actual = noted.note)
            assertEquals(expected = revision, actual = unchanged)
            assertNull(actual = SampleLibrary.document(app = app, id = document.id).note)
        }
    }

    @Test
    fun `the keep-until date is saved, an invalid date is refused and an empty one removes it`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)

            details.saveKeepUntil(document = document, typed = " 2030-01-31 ")
            app.idle()
            val kept = SampleLibrary.document(app = app, id = document.id)
            details.saveKeepUntil(document = kept, typed = "2030-02-30")
            app.idle()
            val refused = app.session.state.value.message
            val afterRefusal = SampleLibrary.document(app = app, id = document.id).retention
            details.saveKeepUntil(document = kept, typed = "")
            app.idle()

            assertEquals(expected = LocalDate.parse("2030-01-31"), actual = kept.retention?.keepUntil)
            assertEquals(expected = UiMessageModel(text = DateInput.PROBLEM, isError = true), actual = refused)
            assertEquals(expected = kept.retention, actual = afterRefusal)
            assertNull(actual = SampleLibrary.document(app = app, id = document.id).retention)
        }
    }

    @Test
    fun `a done retention stays done when its date is removed`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val details = detailsOf(app = app)
            details.saveKeepUntil(document = document, typed = "2020-01-31")
            app.idle()
            app.services.documents.settleRetention(documentIds = listOf(document.id))

            details.saveKeepUntil(document = SampleLibrary.document(app = app, id = document.id), typed = "")
            app.idle()

            val retention = SampleLibrary.document(app = app, id = document.id).retention
            assertNull(actual = retention?.keepUntil)
            assertNotNull(actual = retention?.settledAt)
        }
    }

    @Test
    fun `a category keeps documents for 1 to 100 years and other input is refused`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            app.services.documents.editCatalog(edit = CatalogEdit.AddCategory(name = "Invoices", parentId = null))
            val category = SampleLibrary.catalog(app = app).categories.values.single()
            val details = detailsOf(app = app)

            details.saveCategoryYears(categoryId = category.id, typed = " 10 ")
            app.idle()
            val saved = SampleLibrary.catalog(app = app).categories.getValue(key = category.id).retentionYears
            val refusals = listOf("0", "101", "ten").map { typed ->
                details.saveCategoryYears(categoryId = category.id, typed = typed)
                app.idle()
                app.session.state.value.message
            }
            val afterRefusals = SampleLibrary.catalog(app = app).categories.getValue(key = category.id).retentionYears
            details.saveCategoryYears(categoryId = category.id, typed = "")
            app.idle()

            val refused = UiMessageModel(text = "Write a number of years from 1 to 100", isError = true)
            assertEquals(expected = 10, actual = saved)
            assertEquals(expected = listOf(refused, refused, refused), actual = refusals)
            assertEquals(expected = 10, actual = afterRefusals)
            assertNull(actual = SampleLibrary.catalog(app = app).categories.getValue(key = category.id).retentionYears)
        }
    }

    @Test
    fun `a rule from a document needs a correspondent and a category or tags`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            setCorrespondent(app = app, document = document, correspondent = "Sample Utility")
            val details = detailsOf(app = app)

            details.createRule(document = SampleLibrary.document(app = app, id = document.id))
            app.idle()

            assertTrue(actual = SampleLibrary.catalog(app = app).settings.rules.isEmpty())
            assertEquals(
                expected = UiMessageModel(
                    text = "Give the document a correspondent and a category or tags first",
                    isError = true,
                ),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `a rule from a document proposes its tags for its correspondent, who becomes known once`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            setCorrespondent(app = app, document = document, correspondent = "Sample Utility")
            val energy = setOf(TagNameModel.of(raw = "energy"))
            app.services.documents.editCatalog(edit = CatalogEdit.AddTags(documentId = document.id, tags = energy))
            val details = detailsOf(app = app)

            repeat(times = 2) {
                details.createRule(document = SampleLibrary.document(app = app, id = document.id))
                app.idle()
            }

            val settings = SampleLibrary.catalog(app = app).settings
            val rule = settings.rules.first()
            assertEquals(expected = 2, actual = settings.rules.size)
            assertEquals(expected = listOf("Sample Utility"), actual = settings.correspondents.map { it.name })
            assertEquals(
                expected = listOf(RuleCondition.CorrespondentIs(name = "Sample Utility")),
                actual = rule.conditions,
            )
            assertEquals(expected = energy, actual = rule.actions.tags)
            assertEquals(expected = RuleOrigin.MANUAL, actual = rule.origin)
            assertEquals(
                expected = UiMessageModel(text = "Created Rule for Sample Utility"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `the link picker offers the other documents and finds them by name, title or correspondent`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "letters/a.pdf", "letters/b.pdf", "c.pdf")
            val first = documents.getValue(key = "letters/a.pdf")
            setCorrespondent(app = app, document = documents.getValue(key = "c.pdf"), correspondent = "Sample Bakery")
            val details = detailsOf(app = app)

            details.startLink(document = first)
            val offered = details.linkPick.value?.matches.orEmpty().map { it.path }.toSet()
            details.search(query = " LETTERS ")
            val byPath = details.linkPick.value?.matches.orEmpty().map { it.path }
            details.search(query = "bakery")
            val byCorrespondent = details.linkPick.value?.matches.orEmpty().map { it.path }

            assertEquals(expected = setOf("letters/b.pdf", "c.pdf"), actual = offered)
            assertEquals(expected = listOf("letters/b.pdf"), actual = byPath)
            assertEquals(expected = listOf("c.pdf"), actual = byCorrespondent)
        }
    }

    @Test
    fun `a link keeps its trimmed label for both documents and is removed again`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf")
            val first = documents.getValue(key = "a.pdf")
            val second = documents.getValue(key = "b.pdf")
            val details = detailsOf(app = app)

            details.startLink(document = first)
            details.setLinkLabel(label = " Reminder ")
            details.link(other = second.id)
            app.idle()
            val pickAfterLink = details.linkPick.value
            val catalog = SampleLibrary.catalog(app = app)
            val fromFirst = details.linksOf(document = first, catalog = catalog)
            val fromSecond = details.linksOf(document = second, catalog = catalog)
            details.unlink(document = first, other = second.id)
            app.idle()

            assertNull(actual = pickAfterLink)
            assertEquals(
                expected = listOf(
                    LinkCandidateModel(documentId = second.id, name = "b.pdf", path = "b.pdf", label = "Reminder"),
                ),
                actual = fromFirst,
            )
            assertEquals(expected = listOf(first.id), actual = fromSecond.map { it.documentId })
            assertTrue(actual = SampleLibrary.catalog(app = app).links.isEmpty())
        }
    }

    @Test
    fun `a blank label links without a label and a cancelled picker links nothing`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf", "c.pdf")
            val first = documents.getValue(key = "a.pdf")
            val details = detailsOf(app = app)

            details.startLink(document = first)
            details.setLinkLabel(label = "   ")
            details.link(other = documents.getValue(key = "b.pdf").id)
            app.idle()
            details.startLink(document = first)
            details.cancelLink()
            details.link(other = documents.getValue(key = "c.pdf").id)
            app.idle()

            val links = SampleLibrary.catalog(app = app).links
            assertEquals(expected = 1, actual = links.size)
            assertNull(actual = links.single().label)
        }
    }

    @Test
    fun `removing a link that does not exist says so`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.pdf", "b.pdf")
            val details = detailsOf(app = app)

            details.unlink(document = documents.getValue(key = "a.pdf"), other = documents.getValue(key = "b.pdf").id)
            app.idle()

            assertEquals(
                expected = UiMessageModel(text = "The documents are not linked", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `related documents are empty while the index has no entry and without an open library`() {
        UiTest.run { app ->
            val unopened = detailsOf(app = app).related(document = syntheticDocument())
            val document = SampleLibrary.open(app = app, "a.pdf", "b.pdf").getValue(key = "a.pdf")

            val related = detailsOf(app = app).related(document = document)

            assertEquals(expected = emptyList(), actual = unopened)
            assertEquals(expected = emptyList(), actual = related)
        }
    }

    @Test
    fun `without an open library a change reports that no library is open`() {
        UiTest.run { app ->
            val details = detailsOf(app = app)

            details.saveNote(document = syntheticDocument(), note = "Paid")
            app.idle()

            assertEquals(
                expected = UiMessageModel(text = "No library is open", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    private fun detailsOf(app: TestApp): DetailsController {
        return DetailsController(services = app.services, session = app.session)
    }

    private fun formattedFields(app: TestApp, id: DocumentIdModel): Map<FieldKeyModel, String> {
        return SampleLibrary.document(app = app, id = id).fields.mapValues { FieldValues.format(value = it.value) }
    }

    private suspend fun setCorrespondent(app: TestApp, document: DocumentModel, correspondent: String) {
        app.services.documents.editCatalog(
            edit = CatalogEdit.UpdateMetadata(
                documentId = document.id,
                metadata = DocumentMetadataModel(correspondent = correspondent),
            ),
        )
    }

    private fun syntheticDocument(): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = "synthetic"),
            path = RelativePathModel.of(raw = "a.pdf"),
            contentHash = ContentHashModel(value = "a".repeat(n = 64)),
            size = 1,
            addedAt = Instant.parse("2024-01-01T00:00:00Z"),
        )
    }
}
