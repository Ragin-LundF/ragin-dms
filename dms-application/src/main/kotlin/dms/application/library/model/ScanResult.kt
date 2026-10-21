package dms.application.library.model

import dms.application.port.storage.model.StorageResult
import dms.domain.change.DetectedChange
import dms.domain.library.LibraryFolderId

/** Everything one scan of the library folders found. */
data class ScanResult(
    val changes: List<DetectedChange>,
    val scannedFolders: Set<LibraryFolderId>,
    val failedFolders: Map<LibraryFolderId, StorageResult.Failure>,
)
