package dms.ai.prompt

import dms.ai.client.dto.SuggestionDto
import dms.ai.support.Requests
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.taxonomy.TagName
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
        assertEquals(expected = TagName.of(raw = "home"), actual = tags.last().name)
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

    private fun map(answer: SuggestionDto): ClassificationSuggestion {
        return SuggestionMapper.toSuggestion(answer = answer, request = request, model = "sample-model")
    }
}
