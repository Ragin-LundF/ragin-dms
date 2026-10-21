package dms.ui.session.model

import dms.application.library.model.ScanResult
import dms.domain.library.LibraryFolders
import java.time.Instant

/**
 * The state every screen shares. [busy] names the operation that is running; only one runs at a
 * time. [revision] counts finished operations, so screens know when to reload what they show.
 */
data class SessionState(
    val status: LibraryStatus = LibraryStatus.Loading,
    val folders: LibraryFolders? = null,
    val lastScan: ScanResult? = null,
    val lastScanAt: Instant? = null,
    val busy: String? = null,
    val message: UiMessage? = null,
    val revision: Long = 0,
)
