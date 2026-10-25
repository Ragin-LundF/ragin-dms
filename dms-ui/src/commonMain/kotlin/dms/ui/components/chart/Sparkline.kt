package dms.ui.components.chart

import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import dms.ui.theme.Spacing

/**
 * A small line of [values] without axes, with a soft area below it and a dot on the latest value.
 * It shows a direction, not exact figures, so it needs at least two values and draws nothing else.
 */
@Composable
fun Sparkline(values: List<Int>, color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier = modifier) {
        if (values.size < 2) {
            return@Canvas
        }
        val dot = Spacing.hairline.toPx() * 2.2f
        val points = pointsOf(values = values, inset = dot)
        val line = Path().apply {
            moveTo(x = points.first().x, y = points.first().y)
            points.drop(n = 1).forEach { lineTo(x = it.x, y = it.y) }
        }
        val area = Path().apply {
            addPath(path = line)
            lineTo(x = points.last().x, y = size.height)
            lineTo(x = points.first().x, y = size.height)
            close()
        }
        drawPath(path = area, color = color.copy(alpha = 0.12f))
        drawPath(path = line, color = color, style = Stroke(width = Spacing.hairline.toPx() * 1.6f))
        drawCircle(color = color, radius = dot, center = points.last())
    }
}

private fun DrawScope.pointsOf(values: List<Int>, inset: Float): List<Offset> {
    val lowest = values.min()
    val range = (values.max() - lowest).coerceAtLeast(minimumValue = 1)
    val width = size.width - inset * 2
    val height = size.height - inset * 2
    return values.mapIndexed { index, value ->
        Offset(
            x = inset + width * index / values.lastIndex,
            y = inset + height * (1f - (value - lowest).toFloat() / range),
        )
    }
}
