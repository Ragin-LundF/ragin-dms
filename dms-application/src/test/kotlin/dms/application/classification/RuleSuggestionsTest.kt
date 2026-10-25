package dms.application.classification

import dms.application.TestLibrary
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedFolderModel
import dms.domain.classification.model.SuggestedTagModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.match.RuleTarget
import dms.domain.rule.match.model.RuleResultModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.math.BigDecimal
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertSame

class RuleSuggestionsTest {

    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
    private val letters = CategoryModel(id = CategoryIdModel(value = "letters"), name = "Letters")
    private val energy = TagNameModel.of(raw = "Energy")
    private val document = TestLibrary.document(id = "doc-1", path = "inbox/scan.pdf", content = "a")
    private val filed = TestLibrary.document(id = "doc-2", path = "home/energy/old.pdf", content = "b")
    private val billRule = MatchingRuleModel(
        id = MatchingRuleIdModel(value = "rule-bills"),
        name = "Energy bills",
        conditions = listOf(RuleCondition.CorrespondentIs(name = "Sample Energy")),
        actions = RuleActionsModel(categoryId = bills.id),
        origin = RuleOrigin.MANUAL,
    )
    private val catalog = CatalogModel(
        libraryId = LibraryIdModel(value = "library"),
        revision = CatalogRevisionModel.INITIAL,
        documents = mapOf(document.id to document, filed.id to filed),
        categories = mapOf(bills.id to bills, letters.id to letters),
        tags = setOf(energy),
        settings = CatalogSettingsModel(rules = listOf(billRule)),
    )
    private val aiSuggestion = ClassificationSuggestionModel(
        documentId = document.id,
        model = "sample-model",
        title = "Bill",
        category = SuggestedCategory.Existing(categoryId = letters.id),
        tags = listOf(SuggestedTagModel(name = TagNameModel.of(raw = "Paper"), isNew = true)),
        correspondent = "AI Correspondent",
        targetFolder = SuggestedFolderModel(path = RelativePathModel.of(raw = "home/letters"), isNew = true),
        targetReason = "Looks like a letter",
        fields = mapOf(AMOUNT to FieldValue.Number(number = BigDecimal.ONE)),
    )

    @Test
    fun `rules that decided nothing leave the suggestion as it is`() {
        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = RuleResultModel.NONE, catalog = catalog)

        assertSame(expected = aiSuggestion, actual = result)
    }

    @Test
    fun `a suggestion of the rules alone carries only their values and names them as its model`() {
        val rules = RuleResultModel(
            correspondent = "Sample Energy",
            categoryId = bills.id,
            sources = mapOf(RuleTarget.CATEGORY to billRule.id),
        )

        val suggestion = RuleSuggestions.of(document = document, rules = rules, catalog = catalog)

        assertEquals(
            expected = ClassificationSuggestionModel(
                documentId = document.id,
                model = ClassificationSuggestionModel.RULES_MODEL,
                category = SuggestedCategory.Existing(categoryId = bills.id),
                correspondent = "Sample Energy",
                ruleNotes = mapOf(RuleTarget.CATEGORY to "Energy bills"),
            ),
            actual = suggestion,
        )
        assertEquals(expected = true, actual = suggestion.byRules)
    }

    @Test
    fun `the values the rules decided replace the ones of the AI`() {
        val rules = RuleResultModel(
            correspondent = "Sample Energy",
            categoryId = bills.id,
            fields = mapOf(AMOUNT to FieldValue.Number(number = BigDecimal.TEN)),
        )

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(expected = SuggestedCategory.Existing(categoryId = bills.id), actual = result.category)
        assertEquals(expected = "Sample Energy", actual = result.correspondent)
        assertEquals(expected = mapOf(AMOUNT to FieldValue.Number(number = BigDecimal.TEN)), actual = result.fields)
        assertEquals(expected = "Bill", actual = result.title)
        assertEquals(expected = "sample-model", actual = result.model)
    }

    @Test
    fun `what the rules left open keeps the values of the AI`() {
        val due = FieldKeyModel(value = "due")
        val rules = RuleResultModel(fields = mapOf(due to FieldValue.Flag(flag = true)))

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(expected = aiSuggestion.category, actual = result.category)
        assertEquals(expected = "AI Correspondent", actual = result.correspondent)
        assertEquals(expected = aiSuggestion.targetFolder, actual = result.targetFolder)
        assertEquals(expected = "Looks like a letter", actual = result.targetReason)
        assertEquals(
            expected = mapOf(AMOUNT to FieldValue.Number(number = BigDecimal.ONE), due to FieldValue.Flag(flag = true)),
            actual = result.fields,
        )
    }

    @Test
    fun `a category the catalog no longer has is ignored together with its rule note`() {
        val rules = RuleResultModel(
            categoryId = CategoryIdModel(value = "removed"),
            correspondent = "Sample Energy",
            sources = mapOf(RuleTarget.CATEGORY to billRule.id, RuleTarget.TAGS to billRule.id),
            tags = setOf(energy),
        )

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(expected = aiSuggestion.category, actual = result.category)
        assertEquals(expected = mapOf(RuleTarget.TAGS to "Energy bills"), actual = result.ruleNotes)
    }

    @Test
    fun `rule tags come first, are united with the AI tags and marked new only when the catalog lacks them`() {
        val rules = RuleResultModel(tags = setOf(TagNameModel.of(raw = "energy"), TagNameModel.of(raw = "Meter")))
        val suggestion = aiSuggestion.copy(
            tags = listOf(
                SuggestedTagModel(name = TagNameModel.of(raw = "ENERGY"), isNew = true),
                SuggestedTagModel(name = TagNameModel.of(raw = "Paper"), isNew = true),
            ),
        )

        val result = RuleSuggestions.over(suggestion = suggestion, rules = rules, catalog = catalog)

        assertEquals(
            expected = listOf("energy" to false, "Meter" to true, "Paper" to true),
            actual = result.tags.map { it.name.value to it.isNew },
        )
    }

    @Test
    fun `a rule folder replaces the AI folder and drops the AI reason`() {
        val rules = RuleResultModel(targetFolder = RelativePathModel.of(raw = "home/energy"))

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(
            expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "home/energy"), isNew = false),
            actual = result.targetFolder,
        )
        assertNull(actual = result.targetReason)
    }

    @Test
    fun `a rule folder without documents is marked new`() {
        val rules = RuleResultModel(targetFolder = RelativePathModel.of(raw = "home/water"))

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(
            expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "home/water"), isNew = true),
            actual = result.targetFolder,
        )
    }

    @Test
    fun `a folder holding only a subfolder with documents is not new`() {
        val rules = RuleResultModel(targetFolder = RelativePathModel.of(raw = "home"))

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(
            expected = SuggestedFolderModel(path = RelativePathModel.of(raw = "home"), isNew = false),
            actual = result.targetFolder,
        )
    }

    @Test
    fun `a document already in a subfolder of the rule folder stays where it is`() {
        val inSubfolder = aiSuggestion.copy(documentId = filed.id)
        val rules = RuleResultModel(targetFolder = RelativePathModel.of(raw = "home"))

        val result = RuleSuggestions.over(suggestion = inSubfolder, rules = rules, catalog = catalog)

        assertNull(actual = result.targetFolder)
        assertNull(actual = result.targetReason)
    }

    @Test
    fun `a rule whose folder is the top level keeps every document in place, whatever the AI suggests`() {
        val rules = RuleResultModel(targetFolder = RelativePathModel.ROOT)

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertNull(actual = result.targetFolder)
        assertNull(actual = result.targetReason)
    }

    @Test
    fun `a rule note names the rule, or its id when the rule is gone`() {
        val gone = MatchingRuleIdModel(value = "rule-gone")
        val rules = RuleResultModel(
            categoryId = bills.id,
            targetFolder = RelativePathModel.of(raw = "home/energy"),
            sources = mapOf(RuleTarget.CATEGORY to billRule.id, RuleTarget.TARGET_FOLDER to gone),
        )

        val result = RuleSuggestions.over(suggestion = aiSuggestion, rules = rules, catalog = catalog)

        assertEquals(
            expected = mapOf(RuleTarget.CATEGORY to "Energy bills", RuleTarget.TARGET_FOLDER to "rule-gone"),
            actual = result.ruleNotes,
        )
    }

    private companion object {
        val AMOUNT = FieldKeyModel(value = "amount")
    }
}
