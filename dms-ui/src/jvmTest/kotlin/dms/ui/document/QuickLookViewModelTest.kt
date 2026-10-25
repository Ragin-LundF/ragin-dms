package dms.ui.document

import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.model.QuickFact
import dms.ui.support.SampleImages
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import dms.ui.session.DateInput
import kotlinx.coroutines.flow.first
import java.math.BigDecimal
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class QuickLookViewModelTest {

    private val suggestion = """
        {"title":"Sample bill","correspondent":"Sample Energy","tags":["Tax"],
         "fileName":"2024-03-15 Sample Energy - Bill.png"}
    """.trimIndent()

    @Test
    fun `Accept all applies exactly the values the card shows, and the suggestion is gone`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = libraryWithOneImage(app = app)
            val pending = pendingSuggestion(app = app, quickLookViewModel = quickLookViewModel, document = document)

            val shown = quickLookViewModel.suggestedValues(
                suggestion = pending,
                catalog = SampleLibrary.catalog(app = app),
            )
            quickLookViewModel.acceptSuggestion(suggestion = pending)
            app.idle()

            assertEquals(
                expected = listOf("title: Sample bill", "correspondent: Sample Energy", "+ Tax"),
                actual = shown,
            )
            val filed = SampleLibrary.document(app = app, id = document.id)
            assertEquals(expected = "Sample bill", actual = filed.metadata.title)
            assertEquals(expected = "Sample Energy", actual = filed.metadata.correspondent)
            assertEquals(expected = setOf(TagNameModel.of(raw = "Tax")), actual = filed.tags)
            assertEquals(expected = "scan-7.png", actual = filed.path.value)
            assertTrue(actual = Files.exists(app.primary.resolve("scan-7.png")))
            assertNull(actual = quickLookViewModel.review.state.value)
            quickLookViewModel.feed.state.first { document.id !in it.suggestions }
        }
    }

    @Test
    fun `nothing changes before Accept all, and Dismiss rejects the suggestion`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = libraryWithOneImage(app = app)
            val pending = pendingSuggestion(app = app, quickLookViewModel = quickLookViewModel, document = document)

            val untouched = SampleLibrary.document(app = app, id = document.id)
            quickLookViewModel.dismissSuggestion(document = document)
            app.idle()
            val after = quickLookViewModel.feed.state.first { document.id !in it.suggestions }

            assertEquals(expected = document.id, actual = pending.documentId)
            assertNull(actual = untouched.metadata.title)
            assertTrue(actual = untouched.tags.isEmpty())
            assertTrue(actual = after.suggestions.isEmpty())
            assertNull(actual = SampleLibrary.document(app = app, id = document.id).metadata.title)
        }
    }

    @Test
    fun `a fact edited in place is saved, the other metadata stays, and the edit ends`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = SampleLibrary.open(app = app, "letters/sample.pdf").values.single()
            quickLookViewModel.saveFact(document = document, fact = QuickFact.TITLE, typed = "Sample letter")
            app.idle()
            quickLookViewModel.quickLook.edit(fact = QuickFact.CORRESPONDENT)

            quickLookViewModel.saveFact(
                document = SampleLibrary.document(app = app, id = document.id),
                fact = QuickFact.CORRESPONDENT,
                typed = "Sample Office",
            )
            app.idle()

            val saved = SampleLibrary.document(app = app, id = document.id)
            assertEquals(expected = "Sample Office", actual = saved.metadata.correspondent)
            assertEquals(expected = "Sample letter", actual = saved.metadata.title)
            assertEquals(expected = document.metadata.documentDate, actual = saved.metadata.documentDate)
            assertNull(actual = quickLookViewModel.quickLook.state.value.editing)
        }
    }

    @Test
    fun `a date that is no date is refused with the date message, and the edit stays open`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = SampleLibrary.open(app = app, "letters/sample.pdf").values.single()
            quickLookViewModel.quickLook.edit(fact = QuickFact.DATE)

            quickLookViewModel.saveFact(document = document, fact = QuickFact.DATE, typed = "15.03.2024")
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertEquals(expected = DateInput.PROBLEM, actual = message.text)
            assertTrue(actual = message.isError)
            assertEquals(expected = QuickFact.DATE, actual = quickLookViewModel.quickLook.state.value.editing)
            assertEquals(
                expected = document.metadata.documentDate,
                actual = SampleLibrary.document(app = app, id = document.id).metadata.documentDate,
            )
        }
    }

    @Test
    fun `an amount typed in place goes into the amount field, and one that is no amount is refused`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = SampleLibrary.open(app = app, "letters/sample.pdf").values.single()
            quickLookViewModel.details.addStandardFields()
            app.idle()

            quickLookViewModel.saveFact(document = document, fact = QuickFact.AMOUNT, typed = "12.50 EUR")
            app.idle()
            val withAmount = SampleLibrary.document(app = app, id = document.id)
            quickLookViewModel.quickLook.edit(fact = QuickFact.AMOUNT)
            quickLookViewModel.saveFact(document = withAmount, fact = QuickFact.AMOUNT, typed = "a lot")
            app.idle()

            val amount = assertIs<FieldValue.Money>(value = withAmount.fields[FieldPresets.AMOUNT.key])
            assertEquals(expected = 0, actual = amount.amount.compareTo(BigDecimal("12.5")))
            assertEquals(expected = CurrencyCodeModel(value = "EUR"), actual = amount.currency)
            assertTrue(actual = assertNotNull(actual = app.session.state.value.message).isError)
            assertEquals(expected = QuickFact.AMOUNT, actual = quickLookViewModel.quickLook.state.value.editing)
            assertEquals(
                expected = withAmount.fields,
                actual = SampleLibrary.document(app = app, id = document.id).fields,
            )
        }
    }

    @Test
    fun `a tag typed in place is added, and one that is too long is refused`() {
        UiTest.run { app ->
            val quickLookViewModel = quickLookViewModelOf(app = app)
            val document = SampleLibrary.open(app = app, "letters/sample.pdf").values.single()

            quickLookViewModel.saveFact(document = document, fact = QuickFact.TAG, typed = "invoice")
            app.idle()
            quickLookViewModel.quickLook.edit(fact = QuickFact.TAG)
            val tooLong = "x".repeat(n = TagNameModel.MAX_LENGTH + 1)
            quickLookViewModel.saveFact(document = document, fact = QuickFact.TAG, typed = tooLong)
            app.idle()

            assertEquals(
                expected = setOf(TagNameModel.of(raw = "invoice")),
                actual = SampleLibrary.document(app = app, id = document.id).tags,
            )
            assertTrue(actual = assertNotNull(actual = app.session.state.value.message).isError)
            assertEquals(expected = QuickFact.TAG, actual = quickLookViewModel.quickLook.state.value.editing)
        }
    }

    /** Lets the AI suggest [suggestion] for [document] and leaves it pending, as closing the review does. */
    private suspend fun pendingSuggestion(
        app: TestApp,
        quickLookViewModel: QuickLookViewModel,
        document: DocumentModel,
    ): ClassificationSuggestionModel {
        app.ai.suggestion = suggestion
        quickLookViewModel.classifyWithAi(document = document)
        app.idle()
        quickLookViewModel.review.close()
        val feed = quickLookViewModel.feed.state.first { document.id in it.suggestions }
        return feed.suggestions.getValue(key = document.id)
    }

    /**
     * Create it before the library opens: its PDF tools probe for the OCR program in the background,
     * and a probe that outlives a short test resumes on the next test's main dispatcher.
     */
    private fun quickLookViewModelOf(app: TestApp): QuickLookViewModel {
        return QuickLookViewModel(services = app.services, session = app.session, platform = app.platform)
    }

    private suspend fun libraryWithOneImage(app: TestApp): DocumentModel {
        app.configure()
        val library = app.libraryViewModel()
        val source = SampleImages.write(file = app.root.resolve("outside/scan-7.png"))
        app.platform.nextFiles = listOf(source.toString())
        library.files.addFilesToTopLevel()
        app.idle()
        return library.state.first { it.documents.isNotEmpty() }.documents.single().document
    }
}
