package dms.domain.restructure

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.domain.restructure.model.RestructurePlanModel
import dms.domain.restructure.template.TemplateParse
import dms.domain.restructure.template.model.PathTemplateModel
import dms.domain.settings.model.InboxSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class RestructurePlannerTest {

    private val tax = CategoryModel(id = CategoryIdModel(value = "tax"), name = "Tax")
    private val template = assertIs<TemplateParse.Parsed>(
        value = PathTemplateModel.parse(raw = "{category}/{year}"),
    ).template

    private fun catalogOf(vararg documents: DocumentModel): CatalogModel {
        val base = TestData.catalogWith().addCategory(category = tax).applied()
        return documents.fold(initial = base) { catalog, document ->
            catalog.addDocument(document = document).applied()
                .updateMetadata(
                    documentId = document.id,
                    metadata = DocumentMetadataModel(documentDate = LocalDate.of(2024, 5, 1)),
                )
                .applied()
        }
    }

    private fun plan(
        catalog: CatalogModel,
        naming: FileNaming = FileNaming.KEEP,
        inbox: InboxSettingsModel = InboxSettingsModel(),
    ): RestructurePlanModel {
        return RestructurePlanner.plan(
            catalog = catalog,
            scope = catalog.documents.values.toList(),
            template = template,
            naming = naming,
            inbox = inbox,
        )
    }

    @Test
    fun `documents move to their template folder and those in place stay`() {
        val loose = TestData.document(id = "loose", path = "loose.pdf", categoryId = tax.id)
        val placed = TestData.document(id = "placed", path = "Tax/2024/placed.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(loose, placed))

        assertEquals(
            expected = listOf(
                PlannedMoveModel(
                    documentId = loose.id,
                    from = loose.path,
                    to = RelativePathModel.of(raw = "Tax/2024/loose.pdf"),
                ),
            ),
            actual = plan.moves,
        )
        assertEquals(expected = listOf(placed.id), actual = plan.unchanged)
    }

    @Test
    fun `documents without the values the template needs are skipped with a reason`() {
        val uncategorized = TestData.document(id = "uncategorized", path = "a.pdf")

        val plan = plan(catalog = catalogOf(uncategorized))

        assertEquals(expected = SkipReason.MISSING_CATEGORY, actual = plan.skipped.single().reason)
    }

    @Test
    fun `a name clash gets a numbered suffix, also against a document that only differs by case`() {
        val first = TestData.document(id = "first", path = "one/scan.pdf", categoryId = tax.id)
        val second = TestData.document(id = "second", path = "two/SCAN.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(first, second))

        assertEquals(
            expected = listOf("Tax/2024/scan.pdf", "Tax/2024/SCAN (2).pdf"),
            actual = plan.moves.map { it.to.value },
        )
        assertEquals(expected = setOf(second.id), actual = plan.renamedForClash)
    }

    @Test
    fun `names that differ only by Unicode normalization clash`() {
        val nfc = TestData.document(id = "nfc", path = "Tax/2024/Grüße.pdf", categoryId = tax.id)
        val nfd = TestData.document(id = "nfd", path = "old/Grüße.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(nfc, nfd))

        assertEquals(expected = listOf("Tax/2024/Grüße (2).pdf"), actual = plan.moves.map { it.to.value })
    }

    @Test
    fun `a target folder that differs only by case takes the existing spelling`() {
        val existing = TestData.document(id = "existing", path = "TAX/2024/kept.pdf", categoryId = tax.id)
        val loose = TestData.document(id = "loose", path = "loose.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(existing, loose))

        assertEquals(expected = listOf("TAX/2024/loose.pdf"), actual = plan.moves.map { it.to.value })
        assertEquals(expected = listOf(existing.id), actual = plan.unchanged)
    }

    @Test
    fun `the naming convention renames to date and title`() {
        val loose = TestData.document(id = "loose", path = "scan_0001.PDF", categoryId = tax.id)
        val catalog = catalogOf(loose).updateMetadata(
            documentId = loose.id,
            metadata = DocumentMetadataModel(title = "Tax return", documentDate = LocalDate.of(2024, 5, 1)),
        ).applied()

        val plan = plan(catalog = catalog, naming = FileNaming.DATE_AND_TITLE)

        assertEquals(expected = "Tax/2024/2024-05-01 Tax return.PDF", actual = plan.moves.single().to.value)
    }

    @Test
    fun `inbox documents are never restructured`() {
        val inboxed = TestData.document(id = "inboxed", path = "Inbox/new.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(inboxed), inbox = InboxSettingsModel(enabled = true))

        assertEquals(expected = SkipReason.IN_INBOX, actual = plan.skipped.single().reason)
    }

    @Test
    fun `unticking moves keeps the rest of the plan`() {
        val first = TestData.document(id = "first", path = "a/x.pdf", categoryId = tax.id)
        val second = TestData.document(id = "second", path = "b/x.pdf", categoryId = tax.id)

        val plan = plan(catalog = catalogOf(first, second)).only { it.from.parent?.value != "a" }

        assertEquals(expected = listOf(second.id), actual = plan.moves.map { it.documentId })
        assertEquals(expected = setOf(RelativePathModel.of(raw = "b")), actual = plan.sourceFolders)
    }
}
