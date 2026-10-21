package dms.storage.platform

import java.nio.file.Path

/** Moves a file or folder of a library folder to a trash, from where the user can restore it. */
fun interface SystemTrash {

    /** Returns `false` if [entry] could not be moved; nothing was deleted then. */
    fun moveToTrash(libraryRoot: Path, entry: Path): Boolean
}
