package dms.domain.rule.learn.model

import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import java.util.Locale

/**
 * A rule the library suggests after the user approved enough documents of one [correspondent] the
 * same way: [documentCount] documents, most of them with [categoryId] and [tags].
 */
data class RuleProposalModel(
    val correspondent: String,
    val categoryId: CategoryIdModel?,
    val tags: Set<TagNameModel>,
    val documentCount: Int,
) {
    init {
        require(value = correspondent.isNotBlank()) { "Correspondent must not be blank" }
        require(value = categoryId != null || tags.isNotEmpty()) { "A proposal needs a category or tags" }
    }

    /** Identifies what is proposed, so a rejected proposal is not proposed again. */
    val signature: String
        get() {
            val tagList = tags.sorted().joinToString(separator = ",") { it.value.lowercase(Locale.ROOT) }
            return "${correspondent.trim().lowercase(Locale.ROOT)}|${categoryId?.value.orEmpty()}|$tagList"
        }
}
