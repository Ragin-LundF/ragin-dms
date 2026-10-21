package dms.storage.scan

import dms.storage.catalog.CatalogFiles
import dms.storage.fs.AtomicFiles
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
        PlatformTrash.FALLBACK_FOLDER,
    )

    fun isIgnored(name: String, atRoot: Boolean): Boolean {
        return name in NAMES ||
            name.startsWith(prefix = "._") ||
            name.startsWith(prefix = AtomicFiles.TEMP_PREFIX) ||
            (atRoot && CatalogFiles.isCatalogFile(name = name))
    }
}
