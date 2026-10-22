package dms.domain.rule.match

import dms.domain.TestData
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RuleMatcherTest {

    private val energy = CorrespondentModel(name = "Sample Energy", aliases = listOf("SE Power"))
    private val bills = CategoryIdModel(value = "bills")
    private val contracts = CategoryIdModel(value = "contracts")
    private val document = TestData.document(id = "doc", path = "inbox/bill-2024-03.pdf")

    private fun rule(
        id: String,
        conditions: List<RuleCondition>,
        actions: RuleActionsModel,
        origin: RuleOrigin = RuleOrigin.LEARNED,
        enabled: Boolean = true,
    ): MatchingRuleModel {
        return MatchingRuleModel(
            id = MatchingRuleIdModel(value = id),
            name = id,
            conditions = conditions,
            actions = actions,
            origin = origin,
            enabled = enabled,
        )
    }

    @Test
    fun `a correspondent is recognized by name or alias as a whole word`() {
        val byAlias = RuleMatcher.apply(
            rules = emptyList(),
            correspondents = listOf(energy),
            document = document,
            text = "Invoice from se power ltd",
        )
        val partOfWord = RuleMatcher.apply(
            rules = emptyList(),
            correspondents = listOf(energy),
            document = document,
            text = "SE Powerful",
        )

        assertEquals(expected = "Sample Energy", actual = byAlias.correspondent)
        assertNull(actual = partOfWord.correspondent)
    }

    @Test
    fun `each condition holds on its own terms`() {
        val byCorrespondent = rule(
            id = "c",
            conditions = listOf(RuleCondition.CorrespondentIs(name = "sample energy")),
            actions = RuleActionsModel(categoryId = bills),
        )
        val allWords = rule(
            id = "all",
            conditions = listOf(RuleCondition.TextContains(words = listOf("meter", "kWh"), requireAll = true)),
            actions = RuleActionsModel(tags = setOf(TagNameModel.of(raw = "Energy"))),
        )
        val anyWord = rule(
            id = "any",
            conditions = listOf(RuleCondition.TextContains(words = listOf("gas", "water"), requireAll = false)),
            actions = RuleActionsModel(tags = setOf(TagNameModel.of(raw = "Utility"))),
        )
        val glob = rule(
            id = "glob",
            conditions = listOf(RuleCondition.FileNameMatches(glob = "BILL-????-*.pdf")),
            actions = RuleActionsModel(targetFolder = RelativePathModel.of(raw = "Home/Bills")),
        )

        val result = RuleMatcher.apply(
            rules = listOf(byCorrespondent, allWords, anyWord, glob),
            correspondents = listOf(energy),
            document = document,
            text = "Sample Energy: meter reading 120 kWh",
        )

        assertEquals(expected = bills, actual = result.categoryId)
        assertEquals(expected = setOf(TagNameModel.of(raw = "Energy")), actual = result.tags)
        assertEquals(expected = RelativePathModel.of(raw = "Home/Bills"), actual = result.targetFolder)
        assertTrue(actual = result.isSufficient)
    }

    @Test
    fun `the more specific rule wins a conflict, then the manual one`() {
        val broad = rule(
            id = "broad",
            conditions = listOf(RuleCondition.TextContains(words = listOf("contract"), requireAll = false)),
            actions = RuleActionsModel(categoryId = bills),
            origin = RuleOrigin.MANUAL,
        )
        val specific = rule(
            id = "specific",
            conditions = listOf(RuleCondition.TextContains(words = listOf("contract", "insurance"), requireAll = true)),
            actions = RuleActionsModel(categoryId = contracts),
        )
        val learned = rule(
            id = "learned",
            conditions = listOf(RuleCondition.FileNameMatches(glob = "*.pdf")),
            actions = RuleActionsModel(targetFolder = RelativePathModel.of(raw = "Learned")),
        )
        val manual = rule(
            id = "manual",
            conditions = listOf(RuleCondition.FileNameMatches(glob = "bill*")),
            actions = RuleActionsModel(targetFolder = RelativePathModel.of(raw = "Manual")),
            origin = RuleOrigin.MANUAL,
        )

        val result = RuleMatcher.apply(
            rules = listOf(broad, specific, learned, manual),
            correspondents = emptyList(),
            document = document,
            text = "insurance contract",
        )

        assertEquals(expected = contracts, actual = result.categoryId)
        assertEquals(expected = RelativePathModel.of(raw = "Manual"), actual = result.targetFolder)
        assertEquals(expected = MatchingRuleIdModel(value = "specific"), actual = result.sources[RuleTarget.CATEGORY])
    }

    @Test
    fun `a disabled rule is not used and the document's own correspondent counts`() {
        val disabled = rule(
            id = "off",
            conditions = listOf(RuleCondition.CorrespondentIs(name = "Own Sender")),
            actions = RuleActionsModel(categoryId = bills),
            enabled = false,
        )
        val named = document.copy(metadata = DocumentMetadataModel(correspondent = "Own Sender"))

        val result = RuleMatcher.apply(
            rules = listOf(disabled),
            correspondents = emptyList(),
            document = named,
            text = "",
        )

        assertEquals(expected = "Own Sender", actual = result.correspondent)
        assertNull(actual = result.categoryId)
        assertFalse(actual = result.isSufficient)
    }
}
