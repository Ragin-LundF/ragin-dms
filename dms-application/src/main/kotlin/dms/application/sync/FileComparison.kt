package dms.application.sync

import dms.domain.catalog.model.CatalogModel
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
import dms.domain.library.model.RelativePathModel

/** Compares the copies of one path in different folders, as far as a scan can tell. */
object FileComparison {

    /**
     * The content hash of a file. A scanner hashes only files that changed since the last scan, so a
     * missing hash of a catalogued file means it still has the catalogued content.
     */
    fun effectiveHash(facts: FileFactsModel, path: RelativePathModel, catalog: CatalogModel): ContentHashModel? {
        return facts.contentHash ?: catalog.findByPath(path = path)?.contentHash
    }

    fun differs(
        first: FileFactsModel,
        second: FileFactsModel,
        path: RelativePathModel,
        catalog: CatalogModel,
    ): Boolean {
        val firstHash = effectiveHash(facts = first, path = path, catalog = catalog)
        val secondHash = effectiveHash(facts = second, path = path, catalog = catalog)
        if (firstHash != null && secondHash != null) {
            return firstHash != secondHash
        }
        return first.size != second.size
    }
}
