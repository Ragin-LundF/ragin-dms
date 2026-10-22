package dms.application.port.storage

import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.RelativePathModel

/**
 * Follows the scan of one folder. A scan first lists the folder, then reads the files one by one;
 * [fileDone] counts up to the total that [counted] announced. Both are called on the scanning thread.
 */
interface ScanListener {

    /** The folder holds [total] files, which are read next. */
    fun counted(total: Int) {
        // Nothing to do by default.
    }

    /**
     * [done] files are read; the last was [path] with [facts]. [hashed] tells whether its content was
     * read in this scan rather than taken from the scan cache.
     */
    fun fileDone(done: Int, path: RelativePathModel, facts: FileFactsModel, hashed: Boolean) {
        // Nothing to do by default.
    }

    companion object {
        val NONE: ScanListener = object : ScanListener {}
    }
}
