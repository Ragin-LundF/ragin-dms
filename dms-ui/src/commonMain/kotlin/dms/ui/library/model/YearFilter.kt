package dms.ui.library.model

/**
 * Documents of one [year], or of that year and every earlier one with [andEarlier]. The year is the
 * one of the document date, or of when the document was added if it has none.
 */
data class YearFilter(
    val year: Int,
    val andEarlier: Boolean = false,
) {
    val label: String
        get() {
            return if (andEarlier) "Until $year" else year.toString()
        }

    fun matches(year: Int): Boolean {
        return if (andEarlier) year <= this.year else year == this.year
    }

    companion object {

        private const val EARLIER = "≤"

        /** A year like `2024`, or `≤2017` for that year and every earlier one, as the year chart names its bars. */
        fun parse(text: String): YearFilter? {
            val year = text.removePrefix(prefix = EARLIER).toIntOrNull() ?: return null
            return YearFilter(year = year, andEarlier = text.startsWith(prefix = EARLIER))
        }
    }
}
