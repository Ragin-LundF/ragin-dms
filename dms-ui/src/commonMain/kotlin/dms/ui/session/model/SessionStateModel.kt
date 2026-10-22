package dms.ui.session.model

import dms.application.library.model.ScanResultModel
import dms.application.port.state.model.PreferencesModel
import dms.domain.library.model.LibraryFoldersModel
import java.time.Instant

/**
 * The state every screen shares. [operations] are the ones running now, at most one per lane.
 * [revision] counts finished operations and the changes a running operation reports, so screens
 * know when to reload what they show.
 */
data class SessionStateModel(
    val status: LibraryStatus = LibraryStatus.Loading,
    val folders: LibraryFoldersModel? = null,
    val lastScan: ScanResultModel? = null,
    val lastScanAt: Instant? = null,
    val operations: List<RunningOperationModel> = emptyList(),
    val preferences: PreferencesModel = PreferencesModel(),
    val message: UiMessageModel? = null,
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
