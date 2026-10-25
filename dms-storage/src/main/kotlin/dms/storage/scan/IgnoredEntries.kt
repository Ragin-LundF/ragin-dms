package dms.storage.scan

import dms.domain.library.TemporaryNames
import dms.storage.catalog.CatalogFiles
import dms.storage.index.FileVectorIndex
import dms.storage.plan.FilePlanStore
import dms.storage.platform.PlatformTrash
import dms.storage.text.FileTextStore

/** Files and folders a scan skips: system clutter, the tool's own files and unfinished writes. */
object IgnoredEntries {

    private val NAMES = setOf(
        ".DS_Store",
        ".Spotlight-V100",
        ".Trashes",
        ".fseventsd",
        "Icon\r",
        "Thumbs.db",
        "desktop.ini",
        FileTextStore.TEXT_FOLDER,
        FilePlanStore.PLANS_FOLDER,
        FileVectorIndex.INDEX_FOLDER,
        PlatformTrash.FALLBACK_FOLDER,
    )

    fun isIgnored(name: String, atRoot: Boolean): Boolean {
        return name in NAMES ||
            name.startsWith(prefix = "._") ||
            TemporaryNames.isTemporary(name = name) ||
            (atRoot && CatalogFiles.isCatalogFile(name = name))
    }
}
