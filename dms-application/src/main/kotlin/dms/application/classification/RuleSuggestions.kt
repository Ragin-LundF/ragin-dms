package dms.application.classification

import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedFolderModel
import dms.domain.classification.model.SuggestedTagModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.match.RuleTarget
import dms.domain.rule.match.model.RuleResultModel

/**
 * What the matching rules add to a suggestion: a suggestion of their own when they decided category
 * and correspondent, or their values laid over the AI's, which they override. Tags are united.
 */
object RuleSuggestions {

    /** A suggestion made by the rules alone, without the AI. */
    fun of(document: DocumentModel, rules: RuleResultModel, catalog: CatalogModel): ClassificationSuggestionModel {
        val base = ClassificationSuggestionModel(
            documentId = document.id,
            model = ClassificationSuggestionModel.RULES_MODEL,
        )
        return over(suggestion = base, rules = rules, catalog = catalog)
    }

    /** [suggestion] with every value the rules decided in place of the AI's. */
    fun over(
        suggestion: ClassificationSuggestionModel,
        rules: RuleResultModel,
        catalog: CatalogModel,
    ): ClassificationSuggestionModel {
        if (rules.isEmpty) {
            return suggestion
        }
        val categoryId = rules.categoryId?.takeIf { it in catalog.categories }
        val ruleTags = rules.tags.map { SuggestedTagModel(name = it, isNew = it !in catalog.tags) }
        return suggestion.copy(
            category = categoryId?.let { SuggestedCategory.Existing(categoryId = it) } ?: suggestion.category,
            tags = (ruleTags + suggestion.tags).distinctBy { it.name },
            correspondent = rules.correspondent ?: suggestion.correspondent,
            fields = suggestion.fields + rules.fields,
            targetFolder = rules.targetFolder?.let { folderOf(path = it, catalog = catalog) }
                ?: suggestion.targetFolder,
            targetReason = if (rules.targetFolder != null) null else suggestion.targetReason,
            ruleNotes = rules.sources.mapValues { (_, id) ->
                catalog.settings.rules.firstOrNull { it.id == id }?.name ?: id.value
            }.filterKeys { it != RuleTarget.CATEGORY || categoryId != null },
        )
    }

    private fun folderOf(path: RelativePathModel, catalog: CatalogModel): SuggestedFolderModel? {
        if (path.isRoot) {
            return null
        }
        val exists = catalog.documents.values.any { it.path.isInside(folder = path) }
        return SuggestedFolderModel(path = path, isNew = !exists)
    }
}
