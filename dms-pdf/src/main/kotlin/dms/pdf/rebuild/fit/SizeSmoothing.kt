package dms.pdf.rebuild.fit

import dms.pdf.rebuild.fit.model.FittedLine
import dms.pdf.rebuild.fit.model.TextRunModel
import dms.pdf.rebuild.font.FontMetrics
import dms.pdf.rebuild.font.FontSet
import kotlin.math.abs

/**
 * A letter uses few font sizes: each size becomes the median of all sizes on the page within ±12 %, and
 * then each run is scaled horizontally to its ink, by 90–110 %.
 */
object SizeSmoothing {

    private const val SMOOTHING = 0.12f
    private const val MIN_SCALE = 90f
    private const val MAX_SCALE = 110f

    fun smooth(lines: List<FittedLine>, fonts: FontSet): List<FittedLine> {
        val sizes = lines.filterIsInstance<FittedLine.Text>().map { it.sizePx }
        return lines.map { line ->
            if (line !is FittedLine.Text) return@map line
            val near = sizes.filter { abs(it - line.sizePx) <= line.sizePx * SMOOTHING }.sorted()
            val size = near[near.size / 2]
            val font = fonts.of(style = line.style)
            val runs = line.runs.map { run ->
                val natural = FontMetrics.widthAt1(font = font, text = run.text) * size
                val scale = TextRunModel.FULL_SCALE * run.ink.width / natural
                run.copy(scalePercent = scale.coerceIn(minimumValue = MIN_SCALE, maximumValue = MAX_SCALE))
            }
            line.copy(sizePx = size, runs = runs)
        }
    }
}
