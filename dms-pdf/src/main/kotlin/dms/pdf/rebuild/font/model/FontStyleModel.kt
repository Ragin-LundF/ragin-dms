package dms.pdf.rebuild.font.model

import dms.domain.rebuild.FontFamily

/** Which typeface a line gets: the AI's family guess plus bold. */
data class FontStyleModel(
    val family: FontFamily,
    val bold: Boolean,
)
