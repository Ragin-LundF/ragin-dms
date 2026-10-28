package dms.pdf.rebuild.fit.model

import java.awt.Rectangle

/** One run of real text: a word, or a whole line when its words could not be matched to the ink. */
data class TextRunModel(
    val text: String,
    val ink: Rectangle,
    val scalePercent: Float = FULL_SCALE,
) {
    companion object {
        const val FULL_SCALE: Float = 100f
    }
}
