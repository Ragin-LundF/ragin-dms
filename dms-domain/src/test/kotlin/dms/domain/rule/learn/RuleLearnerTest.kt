package dms.domain.rule.learn

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RuleLearnerTest {

    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
    private val other = CategoryModel(id = CategoryIdModel(value = "other"), name = "Other")
    private val energy = TagNameModel.of(raw = "Energy")

    private fun catalogOf(
        categories: List<CategoryModel>,
        tags: List<Set<TagNameModel>> = emptyList(),
        classified: Boolean = true,
    ): CatalogModel {
        val base = TestData.catalogWith().addCategory(
            category = bills,
        ).applied().addCategory(category = other).applied()
        return categories.foldIndexed(initial = base) { index, catalog, category ->
            val document = TestData.document(
                id = "d$index",
                path = "bills/$index.pdf",
                categoryId = category.id,
                tags = tags.getOrElse(index = index) { emptySet() },
            )
            val added = catalog.addDocument(document = document).applied()
                .updateMetadata(
                    documentId = document.id,
                    metadata = DocumentMetadataModel(correspondent = "Sample Energy"),
                ).applied()
            if (classified) {
                added.recordClassification(
                    documentId = document.id,
                    record = ClassificationRecordModel(
                        source = ClassificationSource.MANUAL,
                        model = null,
                        confirmedAt = Instant.EPOCH,
                    ),
                ).applied()
            } else {
                added
            }
        }
    }

    @Test
    fun `three approved documents with the same category make a proposal`() {
        val proposals = RuleLearner.proposals(catalog = catalogOf(categories = listOf(bills, bills, bills)))

        assertEquals(
            expected = listOf(
                RuleProposalModel(
                    correspondent = "Sample Energy",
                    categoryId = bills.id,
                    tags = emptySet(),
                    documentCount = 3,
                ),
            ),
            actual = proposals,
        )
    }

    @Test
    fun `two documents, unapproved ones, or a split category make none`() {
        assertTrue(actual = RuleLearner.proposals(catalog = catalogOf(categories = listOf(bills, bills))).isEmpty())
        assertTrue(
            actual = RuleLearner.proposals(
                catalog = catalogOf(categories = listOf(bills, bills, bills), classified = false),
            ).isEmpty(),
        )
        assertTrue(
            actual = RuleLearner.proposals(catalog = catalogOf(categories = listOf(bills, bills, other))).isEmpty(),
        )
    }

    @Test
    fun `tags shared by at least 80 percent are proposed too`() {
        val tags = listOf(setOf(energy), setOf(energy), setOf(energy), setOf(energy), emptySet())
        val proposals = RuleLearner.proposals(
            catalog = catalogOf(categories = listOf(bills, bills, bills, bills, other), tags = tags),
        )

        assertEquals(expected = setOf(energy), actual = proposals.single().tags)
        assertEquals(expected = null, actual = proposals.single().categoryId)
    }

    @Test
    fun `a correspondent that already has a rule is not proposed again`() {
        val catalog = catalogOf(categories = listOf(bills, bills, bills))
        val rule = MatchingRuleModel(
            id = MatchingRuleIdModel(value = "r"),
            name = "Sample Energy",
            conditions = listOf(RuleCondition.CorrespondentIs(name = "sample energy")),
            actions = RuleActionsModel(categoryId = bills.id),
            origin = RuleOrigin.MANUAL,
        )
        val withRule = catalog.changeSettings(settings = catalog.settings.withRule(rule = rule)).applied()

        assertTrue(actual = RuleLearner.proposals(catalog = withRule).isEmpty())
    }
}
