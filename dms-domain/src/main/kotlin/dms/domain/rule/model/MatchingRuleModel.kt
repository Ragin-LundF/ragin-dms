package dms.domain.rule.model

import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin

/**
 * "When a document looks like this, propose that": every [conditions] entry must hold, and the
 * [actions] become part of the suggestion the user reviews. A disabled rule is kept but not used.
 */
data class MatchingRuleModel(
    val id: MatchingRuleIdModel,
    val name: String,
    val conditions: List<RuleCondition>,
    val actions: RuleActionsModel,
    val origin: RuleOrigin,
    val enabled: Boolean = true,
) {
    init {
        require(value = name.isNotBlank()) { "Rule name must not be blank" }
        require(value = conditions.isNotEmpty()) { "A rule needs at least one condition" }
        require(value = !actions.isEmpty) { "A rule needs at least one action" }
    }

    /** More conditions make a rule more specific; a more specific rule wins a conflict. */
    val specificity: Int
        get() {
            return conditions.sumOf { condition ->
                when (condition) {
                    is RuleCondition.TextContains -> if (condition.requireAll) condition.words.size else 1
                    else -> 1
                }
            }
        }
}
