package dms.domain.rule.match.model

import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.match.RuleTarget
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/**
 * What the matching rules propose for one document, with the rule behind each value so the review can
 * say "From rule: …". [correspondent] is the one recognized in the text, if any.
 */
data class RuleResultModel(
    val correspondent: String? = null,
    val categoryId: CategoryIdModel? = null,
    val tags: Set<TagNameModel> = emptySet(),
    val fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
    val targetFolder: RelativePathModel? = null,
    val sources: Map<RuleTarget, MatchingRuleIdModel> = emptyMap(),
) {
    /** `true` if the rules decided both category and correspondent, so the AI is not needed. */
    val isSufficient: Boolean
        get() {
            return categoryId != null && correspondent != null
        }

    val isEmpty: Boolean
        get() {
            return correspondent == null && categoryId == null && tags.isEmpty() && fields.isEmpty() &&
                targetFolder == null
        }

    companion object {
        val NONE: RuleResultModel = RuleResultModel()
    }
}
