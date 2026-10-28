package dms.pdf.rebuild.font

import org.apache.pdfbox.pdmodel.font.PDType0Font

/** Text measures in units of the font size, for fitting a line to its ink. */
object FontMetrics {

    private const val UNITS = 1000f
    private const val DEFAULT_CAP = 700f
    private const val DEFAULT_X_HEIGHT = 500f
    private const val ACCENTED_CAP = 1.22f
    private const val LOW_MARK = 0.4f
    private const val ACCENTED = "ÄÖÜÀÁÂÉÈÊÅ"
    private const val TALL = "bdfhklitjäöüß€$%&/()[]{}|!?\"'#@"
    private const val MIDDLE = "+=<>*~^"

    /** The width of [text] at size 1. */
    fun widthAt1(font: PDType0Font, text: String): Float {
        return font.getStringWidth(text) / UNITS
    }

    /** Whether [font] has a glyph for every character of [text]. */
    fun canEncode(font: PDType0Font, text: String): Boolean {
        return runCatching { font.encode(text) }.isSuccess
    }

    /** How far the ink of [text] reaches above the baseline at size 1, from cap height and x-height. */
    fun inkTop(font: PDType0Font, text: String): Float {
        val cap = (font.fontDescriptor?.capHeight?.takeIf { it > 0 } ?: DEFAULT_CAP) / UNITS
        val xHeight = (font.fontDescriptor?.xHeight?.takeIf { it > 0 } ?: DEFAULT_X_HEIGHT) / UNITS
        return text.maxOf { c ->
            when {
                c in ACCENTED -> cap * ACCENTED_CAP
                c.isUpperCase() || c.isDigit() || c in TALL -> cap
                c.isLetter() || c in MIDDLE -> xHeight
                else -> xHeight * LOW_MARK
            }
        }
    }
}
