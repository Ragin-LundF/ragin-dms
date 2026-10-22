package dms.application.port.storage.model

import dms.domain.library.model.LibraryFolderIdModel
import java.time.Instant

/** A [FolderOperation] that failed for one sync folder and waits to be retried. */
data class PendingOperationModel(
    val id: String,
    val folderId: LibraryFolderIdModel,
    val operation: FolderOperation,
    val recordedAt: Instant,
)
