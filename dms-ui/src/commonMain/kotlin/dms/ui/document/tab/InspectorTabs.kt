package dms.ui.document.tab

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import dms.ui.components.ReasonTooltip
import dms.ui.document.model.InspectorTab
import dms.ui.theme.Spacing

/**
 * The inspector's tabs as text with a line under the selected one. A tab that does not apply to the
 * document, such as Pages for a file that is no PDF, is disabled and says why ([disabled]).
 */
@Composable
fun InspectorTabs(selected: InspectorTab, disabled: Map<InspectorTab, String>, onSelect: (InspectorTab) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.hairline * 2)) {
            InspectorTab.entries.forEach { tab ->
                val reason = disabled[tab]
                ReasonTooltip(reason = reason) {
                    TabLabel(tab = tab, selected = tab == selected, enabled = reason == null) { onSelect(tab) }
                }
            }
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

@Composable
private fun TabLabel(tab: InspectorTab, selected: Boolean, enabled: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Column(
        modifier = Modifier
            .width(intrinsicSize = IntrinsicSize.Max)
            .selectable(selected = selected, enabled = enabled, role = Role.Tab, onClick = onClick),
    ) {
        Text(
            text = tab.label,
            style = MaterialTheme.typography.labelMedium,
            color = when {
                !enabled -> colors.outline
                selected -> colors.onSurface
                else -> colors.onSurfaceVariant
            },
            maxLines = 1,
            softWrap = false,
            modifier = Modifier.padding(horizontal = Spacing.small, vertical = Spacing.small),
        )
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(height = Spacing.hairline * 2)
                .background(color = if (selected) colors.primary else Color.Transparent),
        )
    }
}
