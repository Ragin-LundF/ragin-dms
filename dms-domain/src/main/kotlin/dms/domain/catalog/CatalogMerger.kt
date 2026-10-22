package dms.domain.catalog

import dms.domain.catalog.model.CatalogMergeResultModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.link.model.DocumentLinkModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel

/**
 * Merges two copies of the same catalog, for example the copies of two library folders, or a
 * conflict copy a sync client created when two devices saved at the same time.
 *
 * There is no common ancestor to compare against, so an entry missing on one side cannot be told
 * apart from an entry deleted there. The merge therefore never deletes: it is the union of both
 * copies. Where both copies hold different versions of the same entry, the copy with the higher
 * revision wins (the first one on a tie) and the decision is reported as a conflict.
 *
 * Links are merged as a union of pairs. Field definitions, saved searches and rules are merged per id
 * like categories; correspondents are a union by name; the newer copy's layout wins.
 */
object CatalogMerger {

    fun merge(first: CatalogModel, second: CatalogModel): CatalogMergeResultModel {
        require(value = first.libraryId == second.libraryId) { "Only copies of the same library can be merged" }
        val newer = if (second.revision > first.revision) second else first
        val older = if (newer === first) second else first

        val categoryMerge = mergeCategories(newer = newer.categories, older = older.categories)
        val documentMerge = mergeDocuments(newer = newer.documents, older = older.documents)
        val settingsMerge = mergeSettings(newer = newer.settings, older = older.settings)
        val merged = newer.copy(
            documents = documentMerge.entries,
            categories = categoryMerge.entries,
            tags = newer.tags + older.tags,
            links = mergeLinks(newer = newer.links, older = older.links, documents = documentMerge.entries.keys),
            settings = settingsMerge.settings,
        )
        val conflicts = categoryMerge.conflicts + documentMerge.conflicts + settingsMerge.conflicts
        if (merged == newer) {
            return CatalogMergeResultModel(catalog = newer, conflicts = conflicts)
        }
        return CatalogMergeResultModel(catalog = merged.copy(revision = newer.revision.next()), conflicts = conflicts)
    }

    // Categories only in the older copy cannot form a cycle with the newer copy's tree: nothing in the
    // newer copy can point at an id it does not know.
    private fun mergeCategories(
        newer: Map<CategoryIdModel, CategoryModel>,
        older: Map<CategoryIdModel, CategoryModel>,
    ): PartialMerge<CategoryIdModel, CategoryModel> {
        val conflicts = mutableListOf<CatalogMergeConflict>()
        val entries = newer.toMutableMap()
        older.values.forEach { category ->
            val kept = entries[category.id]
            when {
                kept == null -> entries[category.id] = category
                kept != category -> conflicts += CatalogMergeConflict.CategoryDiffers(kept = kept, discarded = category)
            }
        }
        return PartialMerge(entries = entries, conflicts = conflicts)
    }

    private fun mergeDocuments(
        newer: Map<DocumentIdModel, DocumentModel>,
        older: Map<DocumentIdModel, DocumentModel>,
    ): PartialMerge<DocumentIdModel, DocumentModel> {
        val conflicts = mutableListOf<CatalogMergeConflict>()
        val entries = newer.toMutableMap()
        val ownerByPath = newer.values.associateBy { it.path }.toMutableMap()
        older.values.forEach { document ->
            val sameId = entries[document.id]
            val samePath = ownerByPath[document.path]
            when {
                sameId != null && sameId != document ->
                    conflicts += CatalogMergeConflict.DocumentDiffers(kept = sameId, discarded = document)

                sameId == null && samePath != null ->
                    conflicts += CatalogMergeConflict.PathClash(kept = samePath, discarded = document)

                sameId == null -> {
                    entries[document.id] = document
                    ownerByPath[document.path] = document
                }
            }
        }
        return PartialMerge(entries = entries, conflicts = conflicts)
    }

    /** The union of both copies' links between documents the merged catalog has; the newer label wins. */
    private fun mergeLinks(
        newer: Set<DocumentLinkModel>,
        older: Set<DocumentLinkModel>,
        documents: Set<DocumentIdModel>,
    ): Set<DocumentLinkModel> {
        val olderOnly = older.filterNot { link -> newer.any { it.samePair(other = link) } }
        return (newer + olderOnly).filter { it.first in documents && it.second in documents }.toSet()
    }

    private fun mergeSettings(newer: CatalogSettingsModel, older: CatalogSettingsModel): SettingsMerge {
        val conflicts = mutableListOf<CatalogMergeConflict>()
        val fields = unionById(
            newer = newer.fieldDefinitions,
            older = older.fieldDefinitions,
            id = { it.key },
            conflicts = conflicts,
            conflict = CatalogMergeConflict::FieldDefinitionDiffers,
        )
        val searches = unionById(
            newer = newer.savedSearches,
            older = older.savedSearches,
            id = { it.id },
            conflicts = conflicts,
            conflict = CatalogMergeConflict::SavedSearchDiffers,
        )
        val rules = unionById(
            newer = newer.rules,
            older = older.rules,
            id = { it.id },
            conflicts = conflicts,
            conflict = CatalogMergeConflict::RuleDiffers,
        )
        val settings = newer.copy(
            fieldDefinitions = fields,
            savedSearches = searches,
            rules = rules,
            correspondents = mergeCorrespondents(newer = newer.correspondents, older = older.correspondents),
        )
        return SettingsMerge(settings = settings, conflicts = conflicts)
    }

    /** The newer copy's entries in their order, then the older copy's entries it lacks; clashes reported. */
    private fun <K, V> unionById(
        newer: List<V>,
        older: List<V>,
        id: (V) -> K,
        conflicts: MutableList<CatalogMergeConflict>,
        conflict: (V, V) -> CatalogMergeConflict,
    ): List<V> {
        val byId = newer.associateBy(keySelector = id)
        val added = older.filter { entry ->
            val kept = byId[id(entry)]
            if (kept != null && kept != entry) {
                conflicts += conflict(kept, entry)
            }
            kept == null
        }
        return newer + added
    }

    /** Correspondents by name; the aliases of both copies are kept. */
    private fun mergeCorrespondents(
        newer: List<CorrespondentModel>,
        older: List<CorrespondentModel>,
    ): List<CorrespondentModel> {
        val olderByKey = older.associateBy { it.key }
        val merged = newer.map { correspondent ->
            val other = olderByKey[correspondent.key] ?: return@map correspondent
            correspondent.copy(aliases = (correspondent.aliases + other.aliases).distinct())
        }
        val newerKeys = newer.map { it.key }.toSet()
        return merged + older.filterNot { it.key in newerKeys }
    }

    private class SettingsMerge(
        val settings: CatalogSettingsModel,
        val conflicts: List<CatalogMergeConflict>,
    )

    private class PartialMerge<K, V>(
        val entries: Map<K, V>,
        val conflicts: List<CatalogMergeConflict>,
    )
}
