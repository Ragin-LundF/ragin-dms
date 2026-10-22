package dms.application.port.system

import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

interface Clock {

    fun now(): Instant

    fun today(): LocalDate

    /** The time zone that turns an instant into a day, as [today] does. */
    fun zone(): ZoneId
}
