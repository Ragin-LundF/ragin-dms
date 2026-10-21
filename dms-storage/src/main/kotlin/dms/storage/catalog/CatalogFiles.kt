package dms.storage.catalog

import dms.storage.fs.PathResolver
import java.nio.file.Files
import java.nio.file.Path
import java.util.Locale

/**
 * The catalog file and the conflict copies sync clients create next to it, such as
 * `dms-catalog 2.yaml`, `dms-catalog (conflicted copy).yaml` or `dms-catalog.sync-conflict-….yaml`.
 */
object CatalogFiles {

    const val FILE_NAME: String = "dms-catalog.yaml"

    private const val PREFIX = "dms-catalog"
    private const val EXTENSION = ".yaml"

    /** `true` for the catalog file and all of its conflict copies. */
    fun isCatalogFile(name: String): Boolean {
        val lower = name.lowercase(Locale.ROOT)
        return lower.startsWith(prefix = PREFIX) && lower.endsWith(suffix = EXTENSION)
    }

    fun conflictCopies(root: Path): List<Path> {
        return Files.newDirectoryStream(root).use { entries ->
            entries.filter { entry ->
                val name = PathResolver.nameOf(entry = entry)
                Files.isRegularFile(entry) && isCatalogFile(name = name) && name != FILE_NAME
            }.sortedBy { it.fileName.toString() }
        }
    }
}
