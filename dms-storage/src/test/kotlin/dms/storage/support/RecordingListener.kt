package dms.storage.support

import dms.application.port.storage.ScanListener
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.RelativePathModel

/** Records what a scan reports, as `"<done> <path> <hashed>"` per file; [onFile] runs after each. */
class RecordingListener(private val onFile: () -> Unit = {}) : ScanListener {

    val totals: MutableList<Int> = mutableListOf()
    val files: MutableList<String> = mutableListOf()

    override fun counted(total: Int) {
        totals += total
    }

    override fun fileDone(done: Int, path: RelativePathModel, facts: FileFactsModel, hashed: Boolean) {
        files += "$done ${path.value} $hashed"
        onFile()
    }
}
