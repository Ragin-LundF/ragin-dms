package dms.ui.dashboard.task

import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.task.TaskKey

/** Every answer the dashboard can give to an open task. */
interface TaskHandler {

    fun adopt(change: DetectedChange.NewFile)

    fun restore(change: DetectedChange.MissingFile)

    fun deleteEverywhere(change: DetectedChange.MissingFile)

    /** Makes the version of [change]'s path in [sourceFolderId] the version of every folder. */
    fun keepVersion(change: DetectedChange, sourceFolderId: LibraryFolderId)

    fun keepBoth(change: DetectedChange.ContentConflict, renamedIn: LibraryFolderId)

    fun resolveMove(change: DetectedChange.MovedFile, decision: MoveDecision)

    fun reject(documentId: DocumentId)

    fun retry()

    fun dismiss(key: TaskKey)
}
