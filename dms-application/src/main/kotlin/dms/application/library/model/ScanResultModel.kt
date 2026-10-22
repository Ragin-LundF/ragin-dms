package dms.application.library.model

import dms.application.port.storage.model.StorageResult
import dms.domain.change.DetectedChange
import dms.domain.library.model.LibraryFolderIdModel

/** Everything one scan of the library folders found. */
data class ScanResultModel(
    val changes: List<DetectedChange>,
    val scannedFolders: Set<LibraryFolderIdModel>,
    val failedFolders: Map<LibraryFolderIdModel, StorageResult.Failure>,
)
