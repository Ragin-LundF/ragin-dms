package dms.pdf.rebuild.fit

import dms.pdf.rebuild.image.Mask
import dms.pdf.rebuild.image.PageMasks
import java.awt.Color
import java.awt.Rectangle

/** The colour a line is printed in. */
object InkColor {

    private const val BYTE = 0xFF
    private const val RED_SHIFT = 16
    private const val GREEN_SHIFT = 8

    /**
     * The mean colour of the darker half of the ink in [box]: antialiased edges are lighter than the stroke.
     * Faint print has no pixel that dark, so then the darker half of its [glyph] pixels counts.
     */
    fun of(masks: PageMasks, glyph: Mask, box: Rectangle): Color {
        val pixels = pixelsOf(width = masks.width, box = box).filter { i -> masks.gray[i] < masks.inkThreshold / 2 }
            .ifEmpty {
                pixelsOf(width = masks.width, box = box).filter { i -> glyph.bits[i] }
                    .sortedBy { masks.gray[it] }
                    .let { it.take(n = maxOf(1, it.size / 2)) }
            }
            .let { all -> all.map { i -> masks.page.getRGB(i % masks.width, i / masks.width) } }
        if (pixels.isEmpty()) return Color.BLACK
        return Color(
            pixels.sumOf { it shr RED_SHIFT and BYTE } / pixels.size,
            pixels.sumOf { it shr GREEN_SHIFT and BYTE } / pixels.size,
            pixels.sumOf { it and BYTE } / pixels.size,
        )
    }

    private fun pixelsOf(width: Int, box: Rectangle): List<Int> {
        val columns = box.x until box.x + box.width
        return (box.y until box.y + box.height).flatMap { y -> columns.map { x -> y * width + x } }
    }
}
