package dms.application.port.storage.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * An operation that brings one sync folder in line with the primary folder. Each one can be
 * recorded in the journal and replayed later, because it only refers to what is in the primary.
 */
sealed interface FolderOperation {

    /** Copies [path] from the primary. Replaces an existing file only if [replaceExisting] is set. */
    data class CopyFromPrimary(val path: RelativePathModel, val replaceExisting: Boolean = false) : FolderOperation

    data class CreateFolder(val path: RelativePathModel) : FolderOperation

    data class Move(val from: RelativePathModel, val to: RelativePathModel) : FolderOperation

    data class MoveToTrash(val path: RelativePathModel) : FolderOperation

    /** Copies the stored text of a document from the primary's `.dms-text/`. */
    data class CopyText(val documentId: DocumentIdModel) : FolderOperation

    /** Copies the embeddings of a document from the primary's `.dms-index/`. */
    data class CopyIndex(val documentId: DocumentIdModel) : FolderOperation

    /** Copies the record of an applied restructure from the primary's `.dms-plans/`. */
    data class CopyPlan(val name: String) : FolderOperation

    /** Copies the primary's catalog file as it is at the time of execution. */
    data object SaveCatalog : FolderOperation
}
