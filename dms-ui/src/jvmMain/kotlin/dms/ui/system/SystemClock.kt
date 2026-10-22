package dms.ui.system

import dms.application.port.system.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

/** The clock of this device. */
class SystemClock : Clock {

    override fun now(): Instant {
        return Instant.now()
    }

    override fun today(): LocalDate {
        return LocalDate.now()
    }

    override fun zone(): ZoneId {
        return ZoneId.systemDefault()
    }
}
