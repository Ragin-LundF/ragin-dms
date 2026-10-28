package dms.domain.rebuild.model

import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.KeepReason

/**
 * One printed text line as the AI read it. [legible] is the AI's own certainty about every character.
 * [kept] is set when the line must stay an image for a reason found before fitting it to the ink.
 */
data class LayoutLineModel(
    val box: LayoutBoxModel,
    val text: String,
    val bold: Boolean,
    val family: FontFamily,
    val legible: Boolean,
    val kept: KeepReason? = null,
)
