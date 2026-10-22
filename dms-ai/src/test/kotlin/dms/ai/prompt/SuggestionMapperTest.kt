package dms.ai.prompt

import dms.ai.client.dto.FieldAnswerDto
import dms.ai.client.dto.SuggestionDto
import dms.ai.support.Requests
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedFolderModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import java.math.BigDecimal
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class SuggestionMapperTest {

    private val request = Requests.request()

    @Test
    fun `an existing category is found by name or path, ignoring case`() {
        assertEquals(
            expected = SuggestedCategory.Existing(categoryId = Requests.energy.id),
            actual = map(answer = SuggestionDto(category = "invoices / ENERGY")).category,
        )
        assertEquals(
            expected = SuggestedCategory.Existing(categoryId = Requests.invoices.id),
            actual = map(answer = SuggestionDto(category = "invoices")).category,
        )
    }

    @Test
    fun `an unknown category is new, below its parent if the path names one`() {
        assertEquals(
            expected = SuggestedCategory.New(name = "Water", parentId = Requests.invoices.id),
            actual = map(answer = SuggestionDto(category = "Invoices / Water")).category,
        )
        assertEquals(
            expected = SuggestedCategory.New(name = "Contracts", parentId = null),
            actual = map(answer = SuggestionDto(category = "Contracts")).category,
        )
    }

    @Test
    fun `tags keep the catalog spelling and say whether they are new`() {
        val tags = map(answer = SuggestionDto(tags = listOf("tax", " ", "Home", "home"))).tags

        assertEquals(expected = listOf("Tax", "Home"), actual = tags.map { it.name.value })
        assertEquals(expected = listOf(false, true), actual = tags.map { it.isNew })
        assertEquals(expected = TagNameModel.of(raw = "home"), actual = tags.last().name)
    }

    @Test
    fun `values that do not fit are dropped, not the whole suggestion`() {
        val suggestion = map(
            answer = SuggestionDto(
                title = "  Sample bill ",
                documentDate = "15.01.2026",
                correspondent = "null",
                fileName = "a:b",
                confidence = 1.7,
            ),
        )

        assertEquals(expected = "Sample bill", actual = suggestion.title)
        assertNull(actual = suggestion.documentDate)
        assertNull(actual = suggestion.correspondent)
        assertNull(actual = suggestion.fileName)
        assertEquals(expected = 1.0, actual = suggestion.confidence)
    }

    @Test
    fun `the file name keeps the extension of the file`() {
        val withoutExtension = map(answer = SuggestionDto(fileName = "2026-01-15 Sample"))
        val otherExtension = map(answer = SuggestionDto(fileName = "2026-01-15 Sample.docx"))
        val dated = map(answer = SuggestionDto(documentDate = "2026-01-15"))

        assertEquals(expected = "2026-01-15 Sample.pdf", actual = withoutExtension.fileName)
        assertEquals(expected = "2026-01-15 Sample.pdf", actual = otherExtension.fileName)
        assertEquals(expected = LocalDate.parse("2026-01-15"), actual = dated.documentDate)
    }

    @Test
    fun `a target folder must be in the outline or a new folder flagged as new below one`() {
        val withOutline = Requests.request(outline = Requests.outline)
        fun target(folder: String, isNew: Boolean?): SuggestedFolderModel? {
            val answer = SuggestionDto(
                targetFolder = folder,
                targetFolderIsNew = isNew,
                targetReason = "Same sender\nmore",
            )
            return SuggestionMapper.toSuggestion(answer = answer, request = withOutline, model = "m").targetFolder
        }

        assertEquals(
            expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "Home/Power"), isNew = false),
            actual = target("/Home/Power/", false),
        )
        assertEquals(
            expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "Home/Water"), isNew = true),
            actual = target("Home/Water", true),
        )
        assertNull(actual = target(folder = "Home/Water", isNew = false))
        assertNull(actual = target(folder = "Away/Water", isNew = true))
        assertNull(actual = target(folder = "Inbox/Sorted", isNew = true))
        assertNull(actual = target(folder = "Home/Wa:ter", isNew = true))
        assertEquals(
            expected = "Same sender",
            actual = SuggestionMapper.toSuggestion(
                answer = SuggestionDto(targetFolder = "Home", targetReason = "Same sender\nmore"),
                request = withOutline,
                model = "m",
            ).targetReason,
        )
    }

    @Test
    fun `field values must be defined and fit their type, a missing currency takes the default`() {
        val withFields = Requests.request(fieldDefinitions = FieldPresets.ALL)
        val answer = SuggestionDto(
            fields = listOf(
                FieldAnswerDto(key = "amount", value = "1.234,50"),
                FieldAnswerDto(key = "due", value = "next week"),
                FieldAnswerDto(key = "cancel-by", value = "2026-09-30"),
                FieldAnswerDto(key = "unknown", value = "x"),
                FieldAnswerDto(key = "warranty-until", value = null),
            ),
        )

        val fields = SuggestionMapper.toSuggestion(answer = answer, request = withFields, model = "m").fields

        assertEquals(
            expected = mapOf(
                FieldPresets.AMOUNT.key to FieldValue.Money(
                    amount = BigDecimal("1234.50"),
                    currency = CurrencyCodeModel(value = "EUR"),
                ),
                FieldPresets.CANCEL_BY.key to FieldValue.Date(date = LocalDate.of(2026, 9, 30)),
            ),
            actual = fields,
        )
    }

    @Test
    fun `page texts become one transcription with a form feed between pages`() {
        val paged = map(answer = SuggestionDto(transcription = "all", pages = listOf("first", " second ")))
        val whole = map(answer = SuggestionDto(transcription = "all", pages = listOf(" ")))

        assertEquals(expected = "first\u000Csecond", actual = paged.transcription)
        assertEquals(expected = "all", actual = whole.transcription)
    }

    @Test
    fun `without an outline no target folder is suggested`() {
        assertNull(actual = map(answer = SuggestionDto(targetFolder = "Home", targetFolderIsNew = false)).targetFolder)
    }

    private fun map(answer: SuggestionDto): ClassificationSuggestionModel {
        return SuggestionMapper.toSuggestion(answer = answer, request = request, model = "sample-model")
    }
}
