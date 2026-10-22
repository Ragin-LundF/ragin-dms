package dms.ui.dashboard.task

import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.task.OpenTask
import dms.domain.task.model.TaskKeyModel

/** Every answer the dashboard can give to an open task. */
interface TaskHandler {

    fun adopt(change: DetectedChange.NewFile)

    /** Asks first, then moves a file the catalog does not know to the trash where it was found. */
    fun trashNewFile(change: DetectedChange.NewFile)

    fun restore(change: DetectedChange.MissingFile)

    fun deleteEverywhere(change: DetectedChange.MissingFile)

    /** Makes the version of [change]'s path in [sourceFolderId] the version of every folder. */
    fun keepVersion(change: DetectedChange, sourceFolderId: LibraryFolderIdModel)

    fun keepBoth(change: DetectedChange.ContentConflict, renamedIn: LibraryFolderIdModel)

    fun resolveMove(change: DetectedChange.MovedFile, decision: MoveDecision)

    /** Asks first, then moves a new file that copies a catalogued document to the trash where it was found. */
    fun trashDuplicateFile(change: DetectedChange.NewFile)

    /** Asks first, then keeps [keep] of [task]'s copies and moves the others to the trash everywhere. */
    fun keepDuplicate(task: OpenTask.ResolveDuplicates, keep: DocumentIdModel)

    /** Opens the suggestion for review: edit, approve field by field, or reject. */
    fun review(suggestion: ClassificationSuggestionModel)

    fun reject(documentId: DocumentIdModel)

    fun retry()

    /** Opens the file at [path] in [folderId] in its default application, to see what a task is about. */
    fun open(folderId: LibraryFolderIdModel, path: RelativePathModel)

    fun dismiss(key: TaskKeyModel)
}
