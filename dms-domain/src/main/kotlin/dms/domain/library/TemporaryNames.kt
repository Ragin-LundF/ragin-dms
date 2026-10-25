package dms.domain.library

import dms.domain.library.model.RelativePathModel

/**
 * The names of the tool's unfinished writes. Every module that writes a temporary file uses them, so the
 * scanner skips them and the user can recognize and remove leftovers.
 */
object TemporaryNames {

    const val PREFIX: String = ".dms-tmp-"

    fun isTemporary(name: String): Boolean {
        return name.startsWith(prefix = PREFIX)
    }

    /**
     * A temporary name next to [path], made unique by [uniqueId]. It keeps [path]'s extension, so tools
     * that go by the extension still recognize the file.
     */
    fun siblingOf(path: RelativePathModel, uniqueId: String): RelativePathModel {
        val extension = path.fileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val suffix = if (extension.isEmpty()) "" else ".$extension"
        return (path.parent ?: RelativePathModel.ROOT).resolve(name = "$PREFIX$uniqueId$suffix")
    }
}
