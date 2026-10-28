package dms.pdf.rebuild.fit

import dms.pdf.rebuild.image.Mask
import java.awt.Rectangle
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/**
 * Moves the AI's box of a line onto the glyph ink near it: the band of inked rows that overlaps the box
 * most, widened by accents, i-dots and descenders, narrowed to the inked columns. The AI's numbers are only
 * a hint where to look.
 */
object Snap {

    private const val MIN_HEIGHT = 8
    private const val MARGIN_SHARE = 100
    private const val DARK_SHARE = 200
    private const val MAX_GAP = 2
    private const val NEAR_PERCENT = 35
    private const val THIN_TENTHS = 6
    private const val INSIDE_TENTHS = 3
    private const val MIN_HEIGHT_RATIO = 0.4
    private const val MAX_HEIGHT_RATIO = 2.5
    private const val MAX_CENTRE_SHIFT = 0.8

    /** The glyph ink of the line near [predicted], or `null` if there is none or it is far off. */
    fun snap(glyph: Mask, predicted: Rectangle): Rectangle? {
        val height = max(predicted.height, MIN_HEIGHT)
        val x0 = max(0, predicted.x - height / 2 - glyph.width / MARGIN_SHARE)
        val x1 = min(glyph.width, predicted.x + predicted.width + height / 2 + glyph.width / MARGIN_SHARE)
        val y0 = max(0, predicted.y - height * 3 / 2)
        val y1 = min(glyph.height, predicted.y + predicted.height + height * 3 / 2)
        val bands = bands(glyph = glyph, columns = x0 until x1, rows = y0 until y1)
        val best = bands.maxByOrNull { common(band = it, box = predicted) } ?: return null
        if (common(band = best, box = predicted) <= 0) return null
        val rows = widen(best = best, bands = bands, predicted = predicted)
        val columns = (x0 until x1).filter { x -> rows.any { y -> glyph[x, y] } }
        if (columns.isEmpty()) return null
        val width = columns.last() - columns.first() + 1
        val box = Rectangle(columns.first(), rows.first, width, rows.last - rows.first + 1)
        val heightRatio = box.height.toDouble() / predicted.height
        val centreShift = abs(box.centerY - predicted.centerY) / predicted.height
        val plausible = heightRatio in MIN_HEIGHT_RATIO..MAX_HEIGHT_RATIO && centreShift < MAX_CENTRE_SHIFT
        return box.takeIf { plausible }
    }

    /** Runs of rows with enough ink, allowing gaps of up to two rows. */
    private fun bands(glyph: Mask, columns: IntRange, rows: IntRange): List<IntRange> {
        val minDark = max(2, (columns.last - columns.first + 1) / DARK_SHARE)
        val bands = mutableListOf<IntRange>()
        var start = -1
        var gap = 0
        for (y in rows) {
            if (columns.count { x -> glyph[x, y] } >= minDark) {
                if (start < 0) start = y
                gap = 0
            } else if (start >= 0) {
                gap++
            }
            if (gap > MAX_GAP) {
                bands += start..(y - gap)
                start = -1
                gap = 0
            }
        }
        if (start >= 0) bands += start..(rows.last - gap)
        return bands
    }

    /** Accents, i-dots and descenders can form their own thin bands just above or below the line. */
    private fun widen(best: IntRange, bands: List<IntRange>, predicted: Rectangle): IntRange {
        var top = best.first
        var bottom = best.last
        bands.sortedBy { abs(it.first - best.first) }.forEach { band ->
            val lineHeight = bottom - top + 1
            val bandHeight = band.last - band.first + 1
            val near = band.first - bottom <= lineHeight * NEAR_PERCENT / 100 &&
                top - band.last <= lineHeight * NEAR_PERCENT / 100
            val thin = bandHeight < lineHeight * THIN_TENTHS / 10
            val mostlyInside = common(band = band, box = predicted) >= bandHeight * INSIDE_TENTHS / 10
            val joins = near && (thin || mostlyInside)
            if (band != best && joins) {
                top = min(top, band.first)
                bottom = max(bottom, band.last)
            }
        }
        return top..bottom
    }

    private fun common(band: IntRange, box: Rectangle): Int {
        return min(band.last, box.y + box.height) - max(band.first, box.y)
    }
}
