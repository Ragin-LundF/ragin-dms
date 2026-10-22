package dms.application.port.pdf.model

/** One page of the new file: page [page] of the source (counted from 0), turned clockwise by [rotation] degrees. */
data class PageChangeModel(
    val page: Int,
    val rotation: Int = 0,
) {
    init {
        require(value = page >= 0) { "Pages count from 0" }
        require(value = rotation % QUARTER == 0) { "Pages turn in quarter turns" }
    }

    private companion object {
        const val QUARTER = 90
    }
}
