package dms.domain.restructure.template

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.template.model.PathTemplateModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class PathTemplateTest {

    private val insurance = CategoryModel(id = CategoryIdModel(value = "insurance"), name = "Insurance")
    private val car = CategoryModel(
        id = CategoryIdModel(value = "car"),
        name = "Car: Policies",
        parentId = insurance.id,
    )
    private val document = TestData.document(id = "policy", path = "inbox/policy.pdf", categoryId = car.id)
    private val catalog = TestData.catalogWith()
        .addCategory(category = insurance).applied()
        .addCategory(category = car).applied()
        .changeSettings(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL)).applied()
        .addDocument(document = document).applied()
        .updateMetadata(
            documentId = document.id,
            metadata = DocumentMetadataModel(documentDate = LocalDate.of(2024, 3, 5), correspondent = "Sample/Insurer"),
        ).applied()

    private fun render(
        template: String,
        catalog: dms.domain.catalog.model.CatalogModel = this.catalog,
    ): TemplateRendering {
        val parsed = assertIs<TemplateParse.Parsed>(value = PathTemplateModel.parse(raw = template)).template
        return parsed.render(document = catalog.documents.getValue(document.id), catalog = catalog)
    }

    private fun folder(template: String): String {
        return assertIs<TemplateRendering.Rendered>(value = render(template = template)).folder.value
    }

    @Test
    fun `the category becomes one folder per level, with invalid characters replaced`() {
        assertEquals(expected = "Insurance/Car- Policies/2024", actual = folder(template = "{category}/{year}/"))
        assertEquals(expected = "Insurance/2024-03", actual = folder(template = "{category.top}/{year}-{month}"))
    }

    @Test
    fun `values never add folders of their own`() {
        assertEquals(expected = "Sample-Insurer", actual = folder(template = "{correspondent}"))
        assertEquals(expected = "Pdf/Archive", actual = folder(template = " /{kind}/Archive/ "))
    }

    @Test
    fun `a missing value uses the fallback, otherwise it is reported`() {
        val undated = catalog.updateMetadata(documentId = document.id, metadata = DocumentMetadataModel()).applied()

        assertEquals(
            expected = TemplateRendering.Rendered(folder = RelativePathModel.of(raw = "Undated")),
            actual = render(template = "{year|Undated}", catalog = undated),
        )
        assertEquals(
            expected = TemplateRendering.Missing(placeholder = PlaceholderName.YEAR),
            actual = render(template = "{category}/{year}", catalog = undated),
        )
    }

    @Test
    fun `field values can be used`() {
        val withField = catalog.setFields(
            documentId = document.id,
            fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = LocalDate.of(2026, 1, 31))),
        ).applied()

        assertEquals(
            expected = TemplateRendering.Rendered(folder = RelativePathModel.of(raw = "Due/2026-01-31")),
            actual = render(template = "Due/{field:due}", catalog = withField),
        )
    }

    @Test
    fun `broken templates are refused with a reason`() {
        val problems = mapOf(
            "" to TemplateProblem.EMPTY,
            "{category" to TemplateProblem.UNCLOSED_BRACE,
            "{colour}" to TemplateProblem.UNKNOWN_PLACEHOLDER,
            "{field:Not A Key}" to TemplateProblem.INVALID_FIELD_KEY,
            "../{year}" to TemplateProblem.INVALID_FOLDER_NAME,
            "a:b/{year}" to TemplateProblem.INVALID_FOLDER_NAME,
        )

        problems.forEach { (raw, problem) ->
            assertEquals(
                expected = TemplateParse.Invalid(problem = problem),
                actual = PathTemplateModel.parse(raw = raw),
                message = raw,
            )
        }
    }
}
