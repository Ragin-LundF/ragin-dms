package dms.pdf.rebuild.fit.model

import dms.domain.rebuild.KeepReason
import dms.pdf.rebuild.font.model.FontStyleModel
import java.awt.Color
import java.awt.Rectangle

/** A line after the checks, in pixels of the page: real text, or pixels of the scan with the reason. */
sealed interface FittedLine {

    val box: Rectangle

    data class Kept(override val box: Rectangle, val reason: KeepReason) : FittedLine

    /** Real text in [runs], standing on [baseline], at [sizePx] pixels in [color]. */
    data class Text(
        override val box: Rectangle,
        val style: FontStyleModel,
        val runs: List<TextRunModel>,
        val baseline: Int,
        val sizePx: Float,
        val color: Color,
    ) : FittedLine
}
