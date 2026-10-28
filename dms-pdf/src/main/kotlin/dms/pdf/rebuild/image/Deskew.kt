package dms.pdf.rebuild.image

import java.awt.Color
import java.awt.RenderingHints
import java.awt.geom.AffineTransform
import java.awt.image.BufferedImage
import kotlin.math.abs
import kotlin.math.roundToInt
import kotlin.math.tan

/**
 * Straightens slightly rotated scans before the AI and the masks see them: text lines only give sharp
 * peaks in the ink-per-row profile when they run horizontally, so the angle with the sharpest profile is
 * the skew of the page. Angles are in degrees, clockwise positive.
 */
object Deskew {

    private const val STEP = 4
    private const val MAX_ANGLE = 15.0
    private const val MIN_ANGLE = 0.15
    private const val COARSE = 0.5
    private const val FINE = 0.05
    private const val MIN_INK_PIXELS = 200
    private const val MIN_INK = 110
    private const val MAX_INK = 170
    private const val MARGIN_X = 20
    private const val MARGIN_Y = 30

    /** The skew of [page], or 0 when it is too small to matter. */
    fun angleOf(page: BufferedImage): Double {
        val angle = measure(page = page)
        return if (abs(angle) < MIN_ANGLE) 0.0 else angle
    }

    /** [page] turned back by [angle], on white. */
    fun rotate(page: BufferedImage, angle: Double): BufferedImage {
        val turned = BufferedImage(page.width, page.height, BufferedImage.TYPE_INT_RGB)
        val graphics = turned.createGraphics()
        try {
            graphics.color = Color.WHITE
            graphics.fillRect(0, 0, page.width, page.height)
            graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BICUBIC)
            val turn = AffineTransform.getRotateInstance(Math.toRadians(-angle), page.width / 2.0, page.height / 2.0)
            graphics.drawImage(page, turn, null)
        } finally {
            graphics.dispose()
        }
        return turned
    }

    /** Coarse search in 0.5° steps, then 0.05° steps around the best one, on a page reduced to a quarter. */
    private fun measure(page: BufferedImage): Double {
        val profile = InkProfile.of(page = page) ?: return 0.0
        val coarse = steps(from = -MAX_ANGLE, to = MAX_ANGLE, step = COARSE).maxBy { profile.score(degrees = it) }
        return steps(from = coarse - COARSE, to = coarse + COARSE, step = FINE).maxBy { profile.score(degrees = it) }
    }

    private fun steps(from: Double, to: Double, step: Double): Sequence<Double> {
        return generateSequence(seed = from) { it + step }.takeWhile { it <= to + step / 2 }
    }

    /** The ink pixels of a reduced page, away from the margins, which often hold scanner shadows and holes. */
    private class InkProfile(val xs: IntArray, val ys: IntArray, val width: Int, val height: Int) {

        private val rows = IntArray(size = height * 2)

        fun score(degrees: Double): Double {
            rows.fill(element = 0)
            val slope = tan(Math.toRadians(degrees))
            for (i in xs.indices) {
                val row = (ys[i] - (xs[i] - width / 2) * slope).roundToInt() + height / 2
                if (row in rows.indices) rows[row]++
            }
            return rows.sumOf { it.toDouble() * it }
        }

        companion object {
            fun of(page: BufferedImage): InkProfile? {
                val width = page.width / STEP
                val height = page.height / STEP
                val gray = IntArray(size = width * height) { i ->
                    Gray.valueOf(rgb = page.getRGB((i % width) * STEP, (i / width) * STEP))
                }
                val limit = Gray.otsu(gray = gray).coerceIn(minimumValue = MIN_INK, maximumValue = MAX_INK)
                val xs = ArrayList<Int>()
                val ys = ArrayList<Int>()
                for (y in height / MARGIN_Y until height - height / MARGIN_Y) {
                    for (x in width / MARGIN_X until width - width / MARGIN_X) {
                        if (gray[y * width + x] < limit) {
                            xs += x
                            ys += y
                        }
                    }
                }
                if (xs.size < MIN_INK_PIXELS) return null
                return InkProfile(xs = xs.toIntArray(), ys = ys.toIntArray(), width = width, height = height)
            }
        }
    }
}
