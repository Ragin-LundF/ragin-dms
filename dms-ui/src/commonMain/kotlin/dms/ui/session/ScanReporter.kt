package dms.ui.session

import dms.application.library.model.ScanProgressModel
import dms.ui.session.model.OperationProgressModel

/**
 * Turns the progress of a scan into the progress bar and the log: one line per folder, its file count,
 * and every file whose content had to be read. Files taken from the scan cache are only counted.
 */
class ScanReporter(
    private val log: SessionLog,
    private val onProgress: (OperationProgressModel) -> Unit,
) {

    private var announced: ScanProgressModel? = null

    fun report(progress: ScanProgressModel) {
        val previous = announced
        val folder = "${progress.folderName} (${progress.folderNumber} of ${progress.folderCount})"
        when {
            previous == null || previous.folderNumber != progress.folderNumber ->
                log.info(text = "Listing the files of $folder")

            previous.total == null && progress.total != null ->
                log.info(text = "Found ${progress.total} files in ${progress.folderName}")

            progress.hashed -> progress.current?.let { log.info(text = "Read ${it.value}") }
        }
        announced = progress
        onProgress(
            OperationProgressModel(
                step = if (progress.total == null) "Listing $folder" else "Checking $folder",
                done = progress.done,
                total = progress.total,
                detail = progress.current?.value,
            ),
        )
    }
}
