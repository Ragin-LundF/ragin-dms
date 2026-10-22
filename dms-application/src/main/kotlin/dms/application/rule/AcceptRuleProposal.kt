package dms.application.rule

import dms.application.catalog.ChangeSettings
import dms.application.common.OperationOutcome
import dms.application.port.system.IdGenerator
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel

/**
 * Turns a proposal the user accepted into a learned rule: documents of the correspondent get its
 * category and tags proposed. The correspondent joins the known correspondents, so it is also
 * recognized in the text of documents that do not name it yet.
 */
class AcceptRuleProposal(
    private val changeSettings: ChangeSettings,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(proposal: RuleProposalModel): OperationOutcome {
        val rule = MatchingRuleModel(
            id = MatchingRuleIdModel(value = ids.newId()),
            name = proposal.correspondent,
            conditions = listOf(RuleCondition.CorrespondentIs(name = proposal.correspondent)),
            actions = RuleActionsModel(categoryId = proposal.categoryId, tags = proposal.tags),
            origin = RuleOrigin.LEARNED,
        )
        return changeSettings(subject = "Rule ${rule.name}") { settings ->
            val known = settings.correspondents.any { it.key == CorrespondentModel(name = proposal.correspondent).key }
            val withCorrespondent = if (known) {
                settings
            } else {
                settings.withCorrespondent(correspondent = CorrespondentModel(name = proposal.correspondent))
            }
            withCorrespondent.withRule(rule = rule)
        }
    }
}
