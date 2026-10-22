package dms.domain.catalog

import dms.domain.document.model.DocumentModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.taxonomy.model.CategoryModel

/** Something [CatalogMerger] had to decide between two catalog copies. The user reviews each one. */
sealed interface CatalogMergeConflict {

    /** The same document differs in both copies; the newer copy's version was kept. */
    data class DocumentDiffers(val kept: DocumentModel, val discarded: DocumentModel) : CatalogMergeConflict

    /** Two different documents claim the same path; the newer copy's document was kept. */
    data class PathClash(val kept: DocumentModel, val discarded: DocumentModel) : CatalogMergeConflict

    /** The same category differs in both copies; the newer copy's version was kept. */
    data class CategoryDiffers(val kept: CategoryModel, val discarded: CategoryModel) : CatalogMergeConflict

    /** The same custom field is defined differently in both copies; the newer copy's definition was kept. */
    data class FieldDefinitionDiffers(val kept: FieldDefinitionModel, val discarded: FieldDefinitionModel) :
        CatalogMergeConflict

    /** The same saved search differs in both copies; the newer copy's version was kept. */
    data class SavedSearchDiffers(val kept: SavedSearchModel, val discarded: SavedSearchModel) : CatalogMergeConflict

    /** The same matching rule differs in both copies; the newer copy's version was kept. */
    data class RuleDiffers(val kept: MatchingRuleModel, val discarded: MatchingRuleModel) : CatalogMergeConflict
}
