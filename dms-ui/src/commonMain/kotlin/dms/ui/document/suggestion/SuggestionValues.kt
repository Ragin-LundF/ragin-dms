package dms.ui.document.suggestion

import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.field.FieldValues
import dms.ui.components.format.DateTexts

/**
 * The values of a suggestion as short chip texts, in the order the review dialog shows them: exactly
 * the values that accepting it unchanged applies, so the file name only with [acceptFileName].
 */
object SuggestionValues {

    fun of(suggestion: ClassificationSuggestionModel, catalog: CatalogModel, acceptFileName: Boolean): List<String> {
        val labels = catalog.settings.fieldDefinitions.associate { it.key to it.label }
        return listOfNotNull(
            suggestion.title?.takeIf { it.isNotBlank() }?.let { "title: $it" },
            suggestion.category?.let { categoryText(category = it, catalog = catalog) },
            suggestion.correspondent?.takeIf { it.isNotBlank() }?.let { "correspondent: $it" },
            suggestion.documentDate?.let { "date: ${DateTexts.absolute(date = it)}" },
            suggestion.fileName?.takeIf { acceptFileName && it.isNotBlank() }?.let { "name: $it" },
            suggestion.targetFolder?.let { "folder: ${it.path.value}" },
        ) + suggestion.tags.map { "+ ${it.name.value}" } +
            suggestion.fields.map { (key, value) ->
                "${labels[key] ?: key.value}: ${FieldValues.format(value = value)}"
            }
    }

    private fun categoryText(category: SuggestedCategory, catalog: CatalogModel): String {
        return when (category) {
            is SuggestedCategory.Existing ->
                "category: ${catalog.categories[category.categoryId]?.name ?: category.categoryId.value}"

            is SuggestedCategory.New -> "new category: ${category.name}"
        }
    }
}
