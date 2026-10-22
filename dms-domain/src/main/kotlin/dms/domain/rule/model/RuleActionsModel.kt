package dms.domain.rule.model

import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/** What a matching rule proposes for a document. Proposals only: the user approves them in the review. */
data class RuleActionsModel(
    val categoryId: CategoryIdModel? = null,
    val tags: Set<TagNameModel> = emptySet(),
    val fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
    val targetFolder: RelativePathModel? = null,
) {
    val isEmpty: Boolean
        get() {
            return categoryId == null && tags.isEmpty() && fields.isEmpty() && targetFolder == null
        }
}
