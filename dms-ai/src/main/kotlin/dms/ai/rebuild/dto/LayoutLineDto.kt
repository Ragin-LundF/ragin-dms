package dms.ai.rebuild.dto

import kotlinx.serialization.Serializable

/** One text line: [bbox] is x0, y0, x1, y1, in 0..1000 or in pixels of the sent image. */
@Serializable
data class LayoutLineDto(
    val bbox: List<Double> = emptyList(),
    val text: String = "",
    val bold: Boolean = false,
    val font: String = "sans",
    val legible: Boolean = false,
)
