package dms.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.settings.model.SettingsSection
import dms.ui.theme.Spacing

/** The settings [sections] as tabs, styled like the app navigation; the [selected] one sits on a surface pill. */
@Composable
fun SettingsSectionList(
    sections: List<SettingsSection>,
    selected: SettingsSection,
    onSelect: (SettingsSection) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .width(width = Spacing.navigationWidth)
            .selectableGroup(),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
    ) {
        sections.forEach { section ->
            SectionEntry(section = section, selected = section == selected, onClick = { onSelect(section) })
        }
    }
}

@Composable
private fun SectionEntry(section: SettingsSection, selected: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = if (selected) colors.surface else Color.Transparent)
            .border(
                width = Spacing.hairline,
                color = if (selected) colors.outlineVariant else Color.Transparent,
                shape = shape,
            )
            .selectable(selected = selected, role = Role.Tab, onClick = onClick)
            .padding(horizontal = Spacing.medium, vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Icon(
            imageVector = section.icon,
            contentDescription = null,
            tint = if (selected) colors.primary else colors.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.icon),
        )
        Text(
            text = section.label,
            style = MaterialTheme.typography.labelLarge,
            color = if (selected) colors.onSurface else colors.onSurfaceVariant,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
