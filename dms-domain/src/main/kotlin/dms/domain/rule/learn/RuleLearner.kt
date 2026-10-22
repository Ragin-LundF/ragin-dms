package dms.domain.rule.learn

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.learn.model.RuleProposalModel
import java.util.Locale

/**
 * Proposes rules from the documents the user approved: a correspondent with at least [MIN_DOCUMENTS]
 * classified documents of which at least 90 % share a category, or tags that at least 80 % share.
 * Correspondents that already have a rule are left out; rejected proposals are the caller's to drop.
 */
object RuleLearner {

    const val MIN_DOCUMENTS: Int = 3
    private const val CATEGORY_SHARE = 0.9
    private const val TAG_SHARE = 0.8

    fun proposals(catalog: CatalogModel): List<RuleProposalModel> {
        val covered = catalog.settings.rules.flatMap { rule ->
            rule.conditions.filterIsInstance<RuleCondition.CorrespondentIs>().map { keyOf(name = it.name) }
        }.toSet()
        return catalog.documents.values
            .filter { it.classification != null && it.metadata.correspondent != null }
            .groupBy { keyOf(name = checkNotNull(value = it.metadata.correspondent)) }
            .filterKeys { it !in covered }
            .filterValues { it.size >= MIN_DOCUMENTS }
            .mapNotNull { (_, documents) -> proposalOf(documents = documents) }
            .sortedBy { it.correspondent.lowercase(Locale.ROOT) }
    }

    private fun proposalOf(documents: List<DocumentModel>): RuleProposalModel? {
        val categoryId = documents.mapNotNull { it.categoryId }.groupingBy { it }.eachCount()
            .maxByOrNull { it.value }
            ?.takeIf { it.value >= documents.size * CATEGORY_SHARE }
            ?.key
        val tags = documents.flatMap { it.tags }.groupingBy { it }.eachCount()
            .filterValues { it >= documents.size * TAG_SHARE }
            .keys
        if (categoryId == null && tags.isEmpty()) {
            return null
        }
        // The spelling most documents use.
        val name = documents.mapNotNull { it.metadata.correspondent }
            .groupingBy { it }
            .eachCount()
            .maxBy { it.value }
            .key
        return RuleProposalModel(
            correspondent = name,
            categoryId = categoryId,
            tags = tags,
            documentCount = documents.size,
        )
    }

    private fun keyOf(name: String): String {
        return name.trim().lowercase(Locale.ROOT)
    }
}
