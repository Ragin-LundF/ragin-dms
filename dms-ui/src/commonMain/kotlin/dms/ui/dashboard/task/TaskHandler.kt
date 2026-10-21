package dms.ui.dashboard.task

import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import dms.domain.task.OpenTask
import dms.domain.task.TaskKey

/** Every answer the dashboard can give to an open task. */
interface TaskHandler {

    fun adopt(change: DetectedChange.NewFile)

    /** Asks first, then moves a file the catalog does not know to the trash where it was found. */
    fun trashNewFile(change: DetectedChange.NewFile)

    fun restore(change: DetectedChange.MissingFile)

    fun deleteEverywhere(change: DetectedChange.MissingFile)

    /** Makes the version of [change]'s path in [sourceFolderId] the version of every folder. */
    fun keepVersion(change: DetectedChange, sourceFolderId: LibraryFolderId)

    fun keepBoth(change: DetectedChange.ContentConflict, renamedIn: LibraryFolderId)

    fun resolveMove(change: DetectedChange.MovedFile, decision: MoveDecision)

    /** Asks first, then moves a new file that copies a catalogued document to the trash where it was found. */
    fun trashDuplicateFile(change: DetectedChange.NewFile)

    /** Asks first, then keeps [keep] of [task]'s copies and moves the others to the trash everywhere. */
    fun keepDuplicate(task: OpenTask.ResolveDuplicates, keep: DocumentId)

    /** Opens the suggestion for review: edit, approve field by field, or reject. */
    fun review(suggestion: ClassificationSuggestion)

    fun reject(documentId: DocumentId)

    fun retry()

    /** Opens the file at [path] in [folderId] in its default application, to see what a task is about. */
    fun open(folderId: LibraryFolderId, path: RelativePath)

    fun dismiss(key: TaskKey)
}
