package dms.application.port.storage.model

import dms.domain.document.DocumentId
import dms.domain.library.RelativePath

/**
 * An operation that brings one sync folder in line with the primary folder. Each one can be
 * recorded in the journal and replayed later, because it only refers to what is in the primary.
 */
sealed interface FolderOperation {

    /** Copies [path] from the primary. Replaces an existing file only if [replaceExisting] is set. */
    data class CopyFromPrimary(val path: RelativePath, val replaceExisting: Boolean = false) : FolderOperation

    data class CreateFolder(val path: RelativePath) : FolderOperation

    data class Move(val from: RelativePath, val to: RelativePath) : FolderOperation

    data class MoveToTrash(val path: RelativePath) : FolderOperation

    /** Copies the stored text of a document from the primary's `.dms-text/`. */
    data class CopyText(val documentId: DocumentId) : FolderOperation

    /** Copies the primary's catalog file as it is at the time of execution. */
    data object SaveCatalog : FolderOperation
}
