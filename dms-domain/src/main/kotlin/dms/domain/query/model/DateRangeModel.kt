package dms.domain.query.model

import java.time.LocalDate

/** Days from [from] to [to], both included; an open end has no limit. */
data class DateRangeModel(
    val from: LocalDate? = null,
    val to: LocalDate? = null,
) {
    init {
        require(value = from == null || to == null || !to.isBefore(from)) { "The range ends before it starts" }
    }

    fun contains(date: LocalDate): Boolean {
        return (from == null || !date.isBefore(from)) && (to == null || !date.isAfter(to))
    }

    companion object {

        private const val LAST_MONTH = 12
        private const val LAST_DAY = 31

        fun year(year: Int): DateRangeModel {
            return DateRangeModel(from = LocalDate.of(year, 1, 1), to = LocalDate.of(year, LAST_MONTH, LAST_DAY))
        }

        /** [year] and every earlier year. */
        fun untilEndOf(year: Int): DateRangeModel {
            return DateRangeModel(to = LocalDate.of(year, LAST_MONTH, LAST_DAY))
        }
    }
}
