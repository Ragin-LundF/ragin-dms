package dms.application.common

import dms.application.port.state.ActivityLog
import dms.application.port.state.model.ActivityEntryModel
import dms.application.port.state.model.ActivityKind
import dms.application.port.system.Clock

/** Writes one line to the activity log, stamped with the current time. */
class ActivityRecorder(
    private val activity: ActivityLog,
    private val clock: Clock,
) {

    suspend fun record(kind: ActivityKind, subject: String) {
        activity.record(entry = ActivityEntryModel(at = clock.now(), kind = kind, subject = subject))
    }
}
