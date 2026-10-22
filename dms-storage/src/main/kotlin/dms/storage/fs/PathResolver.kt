package dms.storage.fs

import dms.domain.library.FileNamePolicy
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.text.Normalizer

/**
 * The one mapping between [RelativePathModel] (NFC, `/`) and file system paths.
 *
 * macOS may report names decomposed (NFD) while the catalog stores them composed (NFC), so existing
 * entries are found by comparing NFC-normalized names, never by string equality alone.
 */
object PathResolver {

    /** The folder's root directory, or `null` if it is not reachable right now. */
    fun root(folder: LibraryFolderModel): Path? {
        val root = Path.of(folder.location.value)
        return root.takeIf { Files.isDirectory(it) }
    }

    /** The existing entry for [path] below [root], or `null` if there is none. */
    fun existing(root: Path, path: RelativePathModel): Path? {
        var current = root
        for (segment in path.segments) {
            current = child(directory = current, name = segment) ?: return null
        }
        return current
    }

    /**
     * Where [path] lives or would be created below [root]: existing folders are matched by NFC name,
     * the remaining segments are used as given.
     */
    fun target(root: Path, path: RelativePathModel): Path {
        var current = root
        var parentExists = true
        for (segment in path.segments) {
            val found = if (parentExists) child(directory = current, name = segment) else null
            parentExists = found != null
            current = found ?: current.resolve(segment)
        }
        return current
    }

    /** An entry in [directory] whose name equals [name] ignoring case and normalization. */
    fun clashing(directory: Path, name: String): Path? {
        if (!Files.isDirectory(directory)) {
            return null
        }
        return Files.newDirectoryStream(directory).use { entries ->
            entries.firstOrNull { FileNamePolicy.isSameName(first = it.fileName.toString(), second = name) }
        }
    }

    /** The NFC form of an entry's name, as the catalog stores it. */
    fun nameOf(entry: Path): String {
        return Normalizer.normalize(entry.fileName.toString(), Normalizer.Form.NFC)
    }

    private fun child(directory: Path, name: String): Path? {
        val direct = directory.resolve(name)
        if (Files.exists(direct, LinkOption.NOFOLLOW_LINKS)) {
            return direct
        }
        if (!Files.isDirectory(directory)) {
            return null
        }
        return Files.newDirectoryStream(directory).use { entries ->
            entries.firstOrNull { nameOf(entry = it) == name }
        }
    }
}
