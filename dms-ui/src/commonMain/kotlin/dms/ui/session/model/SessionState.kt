package dms.ui.session.model

import dms.application.library.model.ScanResult
import dms.application.port.state.model.Preferences
import dms.domain.library.LibraryFolders
import java.time.Instant

/**
 * The state every screen shares. [operations] are the ones running now, at most one per lane.
 * [revision] counts finished operations and the changes a running operation reports, so screens
 * know when to reload what they show.
 */
data class SessionState(
    val status: LibraryStatus = LibraryStatus.Loading,
    val folders: LibraryFolders? = null,
    val lastScan: ScanResult? = null,
    val lastScanAt: Instant? = null,
    val operations: List<RunningOperation> = emptyList(),
    val preferences: Preferences = Preferences(),
    val message: UiMessage? = null,
    val revision: Long = 0,
) {
    /** The library operation that runs now, which every other change of the library waits for. */
    val busy: String?
        get() {
            return busyIn(lane = OperationLane.LIBRARY)
        }

    /** The AI run that runs now, which every other AI run waits for. */
    val aiBusy: String?
        get() {
            return busyIn(lane = OperationLane.AI)
        }

    /** The label of the operation running in [lane], or `null` when the lane is free. */
    fun busyIn(lane: OperationLane): String? {
        return operations.firstOrNull { it.lane == lane }?.label
    }
}
