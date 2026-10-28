package dms.domain.rebuild.model

/**
 * A box on a page in units of 0..1000 of the page width and height, origin top left. No pixel sizes
 * cross the ports; each side converts to its own image.
 */
data class LayoutBoxModel(
    val left: Double,
    val top: Double,
    val right: Double,
    val bottom: Double,
) {
    companion object {
        const val SCALE: Double = 1000.0
    }
}
