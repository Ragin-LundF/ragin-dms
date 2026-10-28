package dms.pdf.rebuild.fit

import dms.pdf.rebuild.image.Mask
import java.awt.Rectangle
import kotlin.math.max

/** Matches the words of a line to the ink: words are separated by the widest gaps between ink columns. */
object WordInk {

    private const val X_HEIGHT_TO_EM = 0.7f
    private const val MIN_GAP_EM = 0.15f
    private const val GAP_LEAD = 1.2f

    /**
     * Ink boxes for [count] words: the count-1 widest gaps above the [baseline], if they are clearly wider
     * than the gaps inside words; `null` if the words cannot be matched.
     */
    fun of(glyph: Mask, box: Rectangle, baseline: Int, count: Int): List<Rectangle>? {
        val area = tighten(glyph = glyph, box = box, baseline = baseline)
        if (count == 1) return listOf(area)
        val gaps = gaps(glyph = glyph, area = area, top = box.y, baseline = baseline)
        if (gaps.size < count - 1) return null
        val sorted = gaps.sortedByDescending { it.last - it.first }
        val chosen = sorted.take(n = count - 1)
        val narrowestChosen = chosen.last().let { it.last - it.first + 1 }
        val widestOther = sorted.getOrNull(index = count - 1)?.let { it.last - it.first + 1 } ?: 0
        val em = (baseline - box.y) / X_HEIGHT_TO_EM
        if (narrowestChosen < em * MIN_GAP_EM || narrowestChosen < widestOther * GAP_LEAD) return null
        val cuts = chosen.sortedBy { it.first }
        var left = area.x
        return (cuts.map { it.first } + (area.x + area.width)).mapIndexed { i, right ->
            val wordBox = Rectangle(left, box.y, right - left, box.height)
            val word = tighten(glyph = glyph, box = wordBox, baseline = baseline)
            left = cuts.getOrNull(index = i)?.let { it.last + 1 } ?: right
            word
        }
    }

    /** [box] narrowed to the ink columns above the [baseline]. */
    fun tighten(glyph: Mask, box: Rectangle, baseline: Int): Rectangle {
        val rows = box.y until max(baseline, box.y + 1)
        val columns = (box.x until box.x + box.width).filter { x -> rows.any { y -> glyph[x, y] } }
        if (columns.isEmpty()) return box
        return Rectangle(columns.first(), box.y, columns.last() - columns.first() + 1, box.height)
    }

    private fun gaps(glyph: Mask, area: Rectangle, top: Int, baseline: Int): List<IntRange> {
        val filled = (area.x until area.x + area.width).map { x -> (top until baseline).any { y -> glyph[x, y] } }
        val gaps = mutableListOf<IntRange>()
        var start = -1
        filled.forEachIndexed { i, on ->
            if (!on && start < 0) start = i
            if (on && start >= 0) {
                gaps += (area.x + start) until (area.x + i)
                start = -1
            }
        }
        return gaps
    }
}
