package dms.ai.rebuild.dto

import kotlinx.serialization.Serializable

/** An area that is no printed text, such as a logo or a stamp. */
@Serializable
data class LayoutGraphicDto(
    val kind: String = "graphic",
    val bbox: List<Double> = emptyList(),
)
