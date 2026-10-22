package dms.application.rule

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.application.classification.model.SuggestOutcome
import dms.application.classification.model.SuggestionApprovalModel
import dms.application.common.OperationOutcome
import dms.application.port.ai.model.DocumentContent
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.match.RuleTarget
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.task.OpenTask
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class RulesTest {

    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
    private val energy = TagNameModel.of(raw = "Energy")

    private suspend fun libraryWithRule(): TestLibrary {
        val library = TestLibrary()
        library.open()
        library.store.commit { it.addCategory(category = bills) }
        val rule = MatchingRuleModel(
            id = MatchingRuleIdModel(value = "rule-1"),
            name = "Sample Energy bills",
            conditions = listOf(RuleCondition.CorrespondentIs(name = "Sample Energy")),
            actions = RuleActionsModel(categoryId = bills.id, tags = setOf(energy)),
            origin = RuleOrigin.MANUAL,
        )
        library.changeSettings(subject = "rule") {
            it.withRule(rule = rule).withCorrespondent(correspondent = CorrespondentModel(name = "Sample Energy"))
        }
        return library
    }

    @Test
    fun `when rules decide category and correspondent the AI is not asked`() {
        runTest {
            val library = libraryWithRule()
            val document = library.add(path = "scan.pdf", content = "a")
            library.reader.content = DocumentContent.Text(text = "Sample Energy - your bill")

            val suggestion = assertIs<SuggestOutcome.Suggested>(
                value = library.suggestClassification(documentId = document.id),
            ).suggestion

            assertTrue(actual = library.classifier.requests.isEmpty())
            assertEquals(expected = ClassificationSuggestionModel.RULES_MODEL, actual = suggestion.model)
            assertEquals(expected = SuggestedCategory.Existing(categoryId = bills.id), actual = suggestion.category)
            assertEquals(expected = "Sample Energy", actual = suggestion.correspondent)
            assertEquals(expected = listOf(energy), actual = suggestion.tags.map { it.name })
            assertEquals(expected = "Sample Energy bills", actual = suggestion.ruleNotes[RuleTarget.CATEGORY])
        }
    }

    @Test
    fun `an approved rule suggestion is recorded as made by the rules`() {
        runTest {
            val library = libraryWithRule()
            val document = library.add(path = "scan.pdf", content = "a")
            library.reader.content = DocumentContent.Text(text = "Sample Energy")
            library.suggestClassification(documentId = document.id)

            library.approveSuggestion(
                documentId = document.id,
                approval = SuggestionApprovalModel(tags = setOf(energy)),
            )

            assertEquals(
                expected = ClassificationSource.RULE,
                actual = library.catalog.documents.getValue(key = document.id).classification?.source,
            )
        }
    }

    @Test
    fun `when rules decide only part, the AI gets them as fixed and they override its answer`() {
        runTest {
            val library = libraryWithRule()
            val other = CategoryModel(id = CategoryIdModel(value = "other"), name = "Other")
            library.store.commit { it.addCategory(category = other) }
            val document = library.add(path = "scan.pdf", content = "a")
            library.reader.content = DocumentContent.Text(text = "no known sender")
            val manual = MatchingRuleModel(
                id = MatchingRuleIdModel(value = "rule-2"),
                name = "Scans",
                conditions = listOf(RuleCondition.FileNameMatches(glob = "scan*")),
                actions = RuleActionsModel(categoryId = bills.id),
                origin = RuleOrigin.MANUAL,
            )
            library.changeSettings(subject = "rule") { it.withRule(rule = manual) }
            library.classifier.result = dms.application.port.ai.model.ClassificationResult.Suggested(
                suggestion = ClassificationSuggestionModel(
                    documentId = document.id,
                    model = "test-model",
                    category = SuggestedCategory.Existing(categoryId = other.id),
                ),
            )

            val suggestion = assertIs<SuggestOutcome.Suggested>(
                value = library.suggestClassification(documentId = document.id),
            ).suggestion

            assertEquals(
                expected = bills.id,
                actual = assertNotNull(actual = library.classifier.requests.single().fixedValues).categoryId,
            )
            assertEquals(expected = SuggestedCategory.Existing(categoryId = bills.id), actual = suggestion.category)
            assertEquals(expected = "test-model", actual = suggestion.model)
        }
    }

    @Test
    fun `a learned proposal becomes a task, and accepting it adds a rule and the correspondent`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.store.commit { it.addCategory(category = bills) }
            repeat(times = 3) { index ->
                classified(library = library, document = library.add(path = "bill-$index.pdf", content = "$index"))
            }

            val proposal = assertNotNull(actual = library.buildDashboard(scan = null)).tasks
                .filterIsInstance<OpenTask.ReviewLearnedRule>().single().proposal
            assertIs<OperationOutcome.Done>(value = library.acceptRuleProposal(proposal = proposal))

            val rule = library.catalog.settings.rules.single()
            assertEquals(expected = RuleOrigin.LEARNED, actual = rule.origin)
            assertEquals(expected = bills.id, actual = rule.actions.categoryId)
            assertEquals(
                expected = listOf("Sample Energy"),
                actual = library.catalog.settings.correspondents.map { it.name },
            )
            assertTrue(
                actual = assertNotNull(actual = library.buildDashboard(scan = null)).tasks
                    .none { it is OpenTask.ReviewLearnedRule },
            )
        }
    }

    @Test
    fun `a rejected proposal is not proposed again`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.store.commit { it.addCategory(category = bills) }
            repeat(times = 3) { index ->
                classified(library = library, document = library.add(path = "bill-$index.pdf", content = "$index"))
            }
            val task = assertNotNull(
                actual = library.buildDashboard(scan = null),
            ).tasks.filterIsInstance<OpenTask.ReviewLearnedRule>().single()

            library.dismissTask(key = task.key)

            assertTrue(
                actual = assertNotNull(actual = library.buildDashboard(scan = null)).tasks
                    .none { it is OpenTask.ReviewLearnedRule },
            )
        }
    }

    private suspend fun classified(library: TestLibrary, document: DocumentModel) {
        library.editCatalog(
            edit = CatalogEdit.UpdateMetadata(
                documentId = document.id,
                metadata = DocumentMetadataModel(correspondent = "Sample Energy"),
            ),
        )
        library.editCatalog(edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = bills.id))
        library.store.commit {
            it.recordClassification(
                documentId = document.id,
                record = ClassificationRecordModel(
                    source = ClassificationSource.MANUAL,
                    model = null,
                    confirmedAt = library.clock.now(),
                ),
            )
        }
    }
}
