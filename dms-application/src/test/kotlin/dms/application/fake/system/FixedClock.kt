package dms.application.fake.system

import dms.application.port.system.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset

class FixedClock(private val instant: Instant = Instant.parse("2024-03-01T09:00:00Z")) : Clock {

    override fun now(): Instant {
        return instant
    }

    override fun today(): LocalDate {
        return LocalDate.ofInstant(instant, ZoneOffset.UTC)
    }
}
