package dms.ui.components.chart

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.model.ChartSliceModel
import dms.ui.theme.Spacing

/** One legend line of [DonutChart]: colour dot, name, count and share. */
@Composable
fun LegendRow(slice: ChartSliceModel, share: Int) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(modifier = Modifier.size(size = 10.dp).background(color = slice.color, shape = CircleShape))
        Spacer(modifier = Modifier.width(width = Spacing.small))
        Text(
            text = slice.label,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.width(width = 72.dp),
            maxLines = 1,
            softWrap = false,
        )
        Text(text = slice.value.toString(), style = MaterialTheme.typography.labelLarge)
        Spacer(modifier = Modifier.width(width = Spacing.small))
        Text(
            text = "$share %",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
