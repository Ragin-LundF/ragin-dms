package dms.domain.settings.model

import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel

/**
 * The library-wide settings that travel with the catalog to every folder and device: custom field
 * definitions, saved searches, matching rules, known correspondents and the [layout].
 */
data class CatalogSettingsModel(
    val fieldDefinitions: List<FieldDefinitionModel> = emptyList(),
    val savedSearches: List<SavedSearchModel> = emptyList(),
    val rules: List<MatchingRuleModel> = emptyList(),
    val correspondents: List<CorrespondentModel> = emptyList(),
    val layout: LibraryLayoutModel = LibraryLayoutModel(),
) {
    init {
        require(value = fieldDefinitions.distinctBy { it.key }.size == fieldDefinitions.size) {
            "Two field definitions share a key"
        }
        require(value = savedSearches.distinctBy { it.id }.size == savedSearches.size) {
            "Two saved searches share an id"
        }
        require(value = rules.distinctBy { it.id }.size == rules.size) { "Two rules share an id" }
        require(value = correspondents.distinctBy { it.key }.size == correspondents.size) {
            "Two correspondents share a name"
        }
    }

    fun field(key: FieldKeyModel): FieldDefinitionModel? {
        return fieldDefinitions.firstOrNull { it.key == key }
    }

    /** Adds [definition], or replaces the definition with the same key in place. */
    fun withField(definition: FieldDefinitionModel): CatalogSettingsModel {
        return copy(
            fieldDefinitions = replaced(list = fieldDefinitions, item = definition) { it.key == definition.key },
        )
    }

    fun withoutField(key: FieldKeyModel): CatalogSettingsModel {
        return copy(fieldDefinitions = fieldDefinitions.filterNot { it.key == key })
    }

    /** Adds [search], or replaces the saved search with the same id in place. */
    fun withSearch(search: SavedSearchModel): CatalogSettingsModel {
        return copy(savedSearches = replaced(list = savedSearches, item = search) { it.id == search.id })
    }

    fun withoutSearch(id: SavedSearchIdModel): CatalogSettingsModel {
        return copy(savedSearches = savedSearches.filterNot { it.id == id })
    }

    /** Adds [rule], or replaces the rule with the same id in place. */
    fun withRule(rule: MatchingRuleModel): CatalogSettingsModel {
        return copy(rules = replaced(list = rules, item = rule) { it.id == rule.id })
    }

    fun withoutRule(id: MatchingRuleIdModel): CatalogSettingsModel {
        return copy(rules = rules.filterNot { it.id == id })
    }

    /** Adds [correspondent], or replaces the one with the same name (ignoring case) in place. */
    fun withCorrespondent(correspondent: CorrespondentModel): CatalogSettingsModel {
        return copy(
            correspondents = replaced(list = correspondents, item = correspondent) { it.key == correspondent.key },
        )
    }

    fun withLayout(layout: LibraryLayoutModel): CatalogSettingsModel {
        return copy(layout = layout)
    }

    private fun <T> replaced(list: List<T>, item: T, same: (T) -> Boolean): List<T> {
        if (list.none(predicate = same)) {
            return list + item
        }
        return list.map { if (same(it)) item else it }
    }

    companion object {
        val EMPTY: CatalogSettingsModel = CatalogSettingsModel()
    }
}
