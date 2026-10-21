package dms.storage.support

import dms.storage.fs.PathResolver
import dms.storage.platform.SystemTrash
import java.nio.file.Files
import java.nio.file.Path

/** A trash in a test directory that remembers what it received; with [accepts] `false` it refuses everything. */
class RecordingTrash(
    private val directory: Path,
    private val accepts: Boolean = true,
) : SystemTrash {

    private val received = linkedMapOf<String, Path>()

    /** Names of the trashed entries, in the order they arrived. */
    val trashed: List<String>
        get() {
            return received.keys.toList()
        }

    override fun moveToTrash(libraryRoot: Path, entry: Path): Boolean {
        if (!accepts) {
            return false
        }
        Files.createDirectories(directory)
        val target = directory.resolve("${received.size}-${entry.fileName}")
        Files.move(entry, target)
        received[PathResolver.nameOf(entry = entry)] = target
        return true
    }

    /** The content of the trashed file that had [name]. */
    fun contentOf(name: String): String? {
        return received[name]?.let { Files.readString(it) }
    }
}
