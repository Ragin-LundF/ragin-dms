package dms.ui.components.chart

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import dms.ui.components.chart.model.ChartSlice

/** The ring: a track, then the slices clockwise from the top with a small gap between them. */
fun DrawScope.drawDonut(
    slices: List<ChartSlice>,
    total: Int,
    progress: Float,
    track: Color,
    strokeWidth: Dp,
    startAngle: Float,
    gapAngle: Float,
    fullCircle: Float,
) {
    val stroke = Stroke(width = strokeWidth.toPx(), cap = StrokeCap.Butt)
    val inset = stroke.width / 2
    val arcSize = Size(width = size.width - stroke.width, height = size.height - stroke.width)
    val topLeft = Offset(x = inset, y = inset)
    drawArc(
        color = track,
        startAngle = 0f,
        sweepAngle = fullCircle,
        useCenter = false,
        topLeft = topLeft,
        size = arcSize,
        style = stroke,
    )
    val visible = slices.filter { it.value > 0 }
    val gap = if (visible.size > 1) gapAngle else 0f
    var start = startAngle
    visible.forEach { slice ->
        val sweep = fullCircle * slice.value / total * progress
        drawArc(
            color = slice.color,
            startAngle = start + gap / 2,
            sweepAngle = (sweep - gap).coerceAtLeast(minimumValue = 0f),
            useCenter = false,
            topLeft = topLeft,
            size = arcSize,
            style = stroke,
        )
        start += sweep
    }
}
