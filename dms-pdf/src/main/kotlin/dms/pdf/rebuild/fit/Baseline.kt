package dms.pdf.rebuild.fit

import dms.pdf.rebuild.image.Mask
import java.awt.Rectangle

/** Where the letters of a line stand. */
object Baseline {

    private const val FROM_SHARE = 0.4

    /** In the lower 60 % of the line, the row after the sharpest drop of ink per row. */
    fun of(glyph: Mask, box: Rectangle): Int {
        val columns = box.x until box.x + box.width
        val counts = (box.y until box.y + box.height).map { y -> columns.count { x -> glyph[x, y] } }
        val from = (box.height * FROM_SHARE).toInt().coerceAtLeast(minimumValue = 1)
        var best = box.height
        var bestDrop = 0
        for (row in from until counts.size) {
            val drop = counts[row - 1] - counts[row]
            if (drop > bestDrop) {
                bestDrop = drop
                best = row
            }
        }
        return box.y + best
    }
}
