package dms.ui.components.chart

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerIcon
import androidx.compose.ui.input.pointer.pointerHoverIcon
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.model.ChartPointModel

/** One bar of [ColumnChart]: it grows in when shown and shows its value while hovered or when [alwaysLabelled]. */
@Composable
fun ColumnBar(
    point: ChartPointModel,
    fraction: Float,
    color: Color,
    alwaysLabelled: Boolean,
    modifier: Modifier,
    onClick: (() -> Unit)? = null,
) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val grown = growIn(target = fraction)
    Column(
        modifier = modifier
            .hoverable(interactionSource = interaction)
            .then(
                other = if (onClick != null) {
                    Modifier.clickable(interactionSource = interaction, indication = null, onClick = onClick)
                        .pointerHoverIcon(icon = PointerIcon.Hand)
                } else {
                    Modifier
                },
            ),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Bottom,
    ) {
        if (hovered || alwaysLabelled) {
            Text(
                text = point.value.toString(),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                softWrap = false,
            )
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(fraction = grown.coerceAtLeast(minimumValue = 0.01f))
                .background(
                    color = if (hovered) color else color.copy(alpha = 0.82f),
                    shape = RoundedCornerShape(topStart = 4.dp, topEnd = 4.dp),
                ),
        )
    }
}
