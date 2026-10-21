package dms.application.port.storage.model

import dms.domain.library.LibraryFolderId
import java.time.Instant

/** A [FolderOperation] that failed for one sync folder and waits to be retried. */
data class PendingOperation(
    val id: String,
    val folderId: LibraryFolderId,
    val operation: FolderOperation,
    val recordedAt: Instant,
)
