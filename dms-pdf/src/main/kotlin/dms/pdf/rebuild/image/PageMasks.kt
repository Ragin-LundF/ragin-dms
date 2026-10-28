package dms.pdf.rebuild.image

import java.awt.image.BufferedImage

/**
 * The masks a page is measured on. [ink] is dark text, [light] also catches faint print and table rules
 * that scan lighter, [rules] are long thin runs in [light], and [glyphInk] is ink without rules, so table
 * lines do not spoil baselines and widths.
 */
class PageMasks(val page: BufferedImage) {

    val width: Int = page.width
    val height: Int = page.height
    val gray: IntArray = Gray.of(image = page)
    val inkThreshold: Int = Gray.otsu(gray = gray).coerceIn(minimumValue = MIN_INK, maximumValue = MAX_INK)
    private val lightThreshold = (inkThreshold + LIGHT_OFFSET).coerceAtMost(maximumValue = MAX_LIGHT)
    val ink: Mask = below(limit = inkThreshold)
    val light: Mask = below(limit = lightThreshold)
    val rules: Mask = Rules.of(mask = light)
    private val nearRules = rules.dilate(radius = 1)
    val glyphInk: Mask = Mask.without(first = ink, second = nearRules)

    /** [light] without rules and without specks of scanner noise, which the lighter threshold lets in. */
    val faintGlyphInk: Mask by lazy {
        val candidate = Mask.without(first = light, second = nearRules)
        val labels = Labels.of(mask = candidate)
        val bits = BooleanArray(size = candidate.bits.size) { i ->
            val id = labels.ids[i]
            id >= 0 && labels.counts[id] >= SPECK
        }
        Mask(width = width, height = height, bits = bits)
    }

    private fun below(limit: Int): Mask {
        return Mask(width = width, height = height, bits = BooleanArray(size = gray.size) { gray[it] < limit })
    }

    private companion object {
        const val MIN_INK = 110
        const val MAX_INK = 170
        const val LIGHT_OFFSET = 60
        const val MAX_LIGHT = 215
        const val SPECK = 6
    }
}
