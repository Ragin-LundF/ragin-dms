package dms.ai.rebuild

import dms.ai.rebuild.dto.LayoutAnswerDto
import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.LayoutGraphicModel
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel

/**
 * Maps the AI's answer to a layout in 0..1000 of the page. The prompt asks for 0..1000, which Qwen3-style
 * models give; Qwen2.5-style models answer in pixels of the image they saw. If any value is above 1000,
 * the whole answer is taken as pixels of the sent [sentWidth] x [sentHeight] image.
 */
object LayoutMapper {

    private const val BOX_SIZE = 4

    fun toLayout(answer: LayoutAnswerDto, sentWidth: Int, sentHeight: Int): PageLayoutModel {
        val lines = answer.lines.filter { it.bbox.size == BOX_SIZE }
        val graphics = answer.graphics.filter { it.bbox.size == BOX_SIZE }
        val pixels = (lines.flatMap { it.bbox } + graphics.flatMap { it.bbox }).any { it > LayoutBoxModel.SCALE }
        val scaleX = if (pixels) LayoutBoxModel.SCALE / sentWidth else 1.0
        val scaleY = if (pixels) LayoutBoxModel.SCALE / sentHeight else 1.0
        return PageLayoutModel(
            lines = lines.map { line ->
                LayoutLineModel(
                    box = boxOf(values = line.bbox, scaleX = scaleX, scaleY = scaleY),
                    text = line.text,
                    bold = line.bold,
                    family = familyOf(font = line.font),
                    legible = line.legible,
                )
            },
            graphics = graphics.map { graphic ->
                val box = boxOf(values = graphic.bbox, scaleX = scaleX, scaleY = scaleY)
                LayoutGraphicModel(kind = graphic.kind, box = box)
            },
        )
    }

    private fun boxOf(values: List<Double>, scaleX: Double, scaleY: Double): LayoutBoxModel {
        val xs = listOf(values[0], values[2]).map { inPage(value = it * scaleX) }
        val ys = listOf(values[1], values[3]).map { inPage(value = it * scaleY) }
        return LayoutBoxModel(left = xs.min(), top = ys.min(), right = xs.max(), bottom = ys.max())
    }

    private fun inPage(value: Double): Double {
        return value.coerceIn(minimumValue = 0.0, maximumValue = LayoutBoxModel.SCALE)
    }

    private fun familyOf(font: String): FontFamily {
        return FontFamily.entries.firstOrNull { it.name.equals(other = font, ignoreCase = true) } ?: FontFamily.SANS
    }
}
