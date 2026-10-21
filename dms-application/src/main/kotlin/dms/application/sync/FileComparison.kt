package dms.application.sync

import dms.domain.catalog.Catalog
import dms.domain.change.FileFacts
import dms.domain.document.ContentHash
import dms.domain.library.RelativePath

/** Compares the copies of one path in different folders, as far as a scan can tell. */
object FileComparison {

    /**
     * The content hash of a file. A scanner hashes only files that changed since the last scan, so a
     * missing hash of a catalogued file means it still has the catalogued content.
     */
    fun effectiveHash(facts: FileFacts, path: RelativePath, catalog: Catalog): ContentHash? {
        return facts.contentHash ?: catalog.findByPath(path = path)?.contentHash
    }

    fun differs(
        first: FileFacts,
        second: FileFacts,
        path: RelativePath,
        catalog: Catalog,
    ): Boolean {
        val firstHash = effectiveHash(facts = first, path = path, catalog = catalog)
        val secondHash = effectiveHash(facts = second, path = path, catalog = catalog)
        if (firstHash != null && secondHash != null) {
            return firstHash != secondHash
        }
        return first.size != second.size
    }
}
