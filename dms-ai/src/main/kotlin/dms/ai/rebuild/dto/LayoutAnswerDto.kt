package dms.ai.rebuild.dto

import kotlinx.serialization.Serializable

/** The AI's answer for one page, as the layout schema asks for it. */
@Serializable
data class LayoutAnswerDto(
    val lines: List<LayoutLineDto> = emptyList(),
    val graphics: List<LayoutGraphicDto> = emptyList(),
)
