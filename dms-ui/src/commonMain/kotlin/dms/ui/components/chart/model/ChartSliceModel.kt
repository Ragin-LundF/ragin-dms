package dms.ui.components.chart.model

import androidx.compose.ui.graphics.Color

/** One part of a donut chart, with the colour of the entity it stands for. */
data class ChartSliceModel(
    val label: String,
    val value: Int,
    val color: Color,
)
