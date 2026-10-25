package dms.ui.document.quicklook

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.ReasonTooltip
import dms.ui.components.surface.LabelChip
import dms.ui.document.edit.CategoryMenu
import dms.ui.document.edit.InlineTextEditor
import dms.ui.document.model.DocumentFactModel
import dms.ui.document.model.QuickFact
import dms.ui.document.model.QuickFactsModel
import dms.ui.library.model.CategoryEntryModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing
import java.util.Locale

private const val LABEL_WEIGHT = 0.3f
private const val VALUE_WEIGHT = 0.7f
private const val TAG_SUGGESTIONS = 6

/**
 * The key facts as labels and values. A click on a value edits it in place ([editing]); the
 * category is picked from a menu, and "+ tag" adds a tag, with the library's tags as suggestions.
 * While the library is busy ([busyReason]) the values only say why they cannot change.
 */
@Composable
fun QuickFactList(
    facts: QuickFactsModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    editing: QuickFact?,
    busyReason: String?,
    onEdit: (QuickFact) -> Unit,
    onSave: (QuickFact, String) -> Unit,
    onCancel: () -> Unit,
    onPickCategory: (CategoryIdModel?) -> Unit,
    onRemoveTag: (TagNameModel) -> Unit,
) {
    val text: @Composable (QuickFact, String, String?) -> Unit = { fact, label, value ->
        TextFact(
            label = label,
            value = value,
            editing = editing == fact,
            busyReason = busyReason,
            onEdit = { onEdit(fact) },
            onSave = { onSave(fact, it) },
            onCancel = onCancel,
        )
    }
    Column(modifier = Modifier.fillMaxWidth()) {
        CategoryFact(
            category = facts.category,
            categories = categories,
            busyReason = busyReason,
            onPick = onPickCategory,
        )
        FactRow(label = "Tags") {
            TagsValue(
                tags = facts.tags,
                allTags = allTags,
                adding = editing == QuickFact.TAG,
                busyReason = busyReason,
                onAdd = { onEdit(QuickFact.TAG) },
                onSave = { onSave(QuickFact.TAG, it) },
                onCancel = onCancel,
                onRemove = onRemoveTag,
            )
        }
        text(QuickFact.CORRESPONDENT, "From", facts.correspondent)
        text(QuickFact.DATE, "Date", facts.documentDate)
        facts.amount?.let { text(QuickFact.AMOUNT, it.label, it.value) }
        facts.due?.let { due ->
            if (editing == QuickFact.DUE || due.value == null) {
                text(QuickFact.DUE, due.label, due.value)
            } else {
                DueFact(due = due, busyReason = busyReason, onEdit = { onEdit(QuickFact.DUE) })
            }
        }
    }
}

@Composable
private fun FactRow(label: String, value: @Composable () -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(vertical = Spacing.xsmall),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(weight = LABEL_WEIGHT),
            )
            Box(modifier = Modifier.weight(weight = VALUE_WEIGHT)) { value() }
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}

@Composable
private fun TextFact(
    label: String,
    value: String?,
    editing: Boolean,
    busyReason: String?,
    onEdit: () -> Unit,
    onSave: (String) -> Unit,
    onCancel: () -> Unit,
) {
    FactRow(label = label) {
        if (editing) {
            InlineTextEditor(initial = value.orEmpty(), description = label, onSave = onSave, onCancel = onCancel)
        } else {
            EditableValue(description = "Edit $label", busyReason = busyReason, onClick = onEdit) {
                ValueText(text = value)
            }
        }
    }
}

@Composable
private fun CategoryFact(
    category: String?,
    categories: List<CategoryEntryModel>,
    busyReason: String?,
    onPick: (CategoryIdModel?) -> Unit,
) {
    var picking by remember { mutableStateOf(value = false) }
    FactRow(label = "Category") {
        Box {
            EditableValue(description = "Edit Category", busyReason = busyReason, onClick = { picking = true }) {
                ValueText(text = category, missing = "None yet")
            }
            CategoryMenu(
                expanded = picking,
                categories = categories,
                onSelect = { id ->
                    picking = false
                    onPick(id)
                },
                onDismiss = { picking = false },
            )
        }
    }
}

@Composable
private fun DueFact(due: DocumentFactModel, busyReason: String?, onEdit: () -> Unit) {
    FactRow(label = due.label) {
        EditableValue(description = "Edit ${due.label}", busyReason = busyReason, onClick = onEdit) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            ) {
                ValueText(text = due.value)
                due.chip?.let { chip ->
                    LabelChip(
                        text = chip,
                        color = if (due.urgent) AppColors.statusWarning else MaterialTheme.colorScheme.outline,
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun TagsValue(
    tags: List<TagNameModel>,
    allTags: List<TagNameModel>,
    adding: Boolean,
    busyReason: String?,
    onAdd: () -> Unit,
    onSave: (String) -> Unit,
    onCancel: () -> Unit,
    onRemove: (TagNameModel) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            tags.forEach { tag ->
                TagChip(tag = tag, enabled = busyReason == null, onRemove = { onRemove(tag) })
            }
            if (!adding) {
                ReasonTooltip(reason = busyReason) {
                    Text(
                        text = "+ tag",
                        style = MaterialTheme.typography.labelMedium,
                        color = colors.onSurfaceVariant,
                        maxLines = 1,
                        softWrap = false,
                        modifier = Modifier
                            .clip(shape = CircleShape)
                            .border(width = Spacing.hairline, color = colors.outlineVariant, shape = CircleShape)
                            .clickable(enabled = busyReason == null, onClick = onAdd)
                            .padding(horizontal = Spacing.small, vertical = Spacing.hairline * 2),
                    )
                }
            }
        }
        if (adding) {
            InlineTextEditor(
                initial = "",
                description = "New tag",
                onSave = onSave,
                onCancel = onCancel,
                suggestionsFor = { typed -> tagSuggestions(typed = typed, tags = tags, allTags = allTags) },
            )
        }
    }
}

@Composable
private fun TagChip(tag: TagNameModel, enabled: Boolean, onRemove: () -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .background(color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f), shape = CircleShape)
            .padding(start = Spacing.small, end = Spacing.xsmall),
    ) {
        Text(
            text = tag.value,
            style = MaterialTheme.typography.labelMedium,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
        )
        Icon(
            imageVector = Icons.Rounded.Close,
            contentDescription = "Remove tag ${tag.value}",
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier
                .size(size = Spacing.large)
                .clip(shape = CircleShape)
                .clickable(enabled = enabled, onClick = onRemove)
                .padding(all = Spacing.hairline * 2),
        )
    }
}

/** A value that edits on click and shows "edit" while hovered; while busy, it says why it cannot. */
@Composable
private fun EditableValue(
    description: String,
    busyReason: String?,
    onClick: () -> Unit,
    content: @Composable () -> Unit,
) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val shape = MaterialTheme.shapes.extraSmall
    val colors = MaterialTheme.colorScheme
    ReasonTooltip(reason = busyReason) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = shape)
                .background(color = if (hovered) colors.surfaceContainer else colors.surface)
                .hoverable(interactionSource = interaction)
                .clickable(enabled = busyReason == null, onClickLabel = description, onClick = onClick)
                .padding(horizontal = Spacing.xsmall),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(modifier = Modifier.weight(weight = 1f)) { content() }
            if (hovered && busyReason == null) {
                Text(
                    text = "edit",
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.outline,
                )
            }
        }
    }
}

@Composable
private fun ValueText(text: String?, missing: String = "—") {
    Text(
        text = text ?: missing,
        style = MaterialTheme.typography.bodySmall,
        color = if (text == null) MaterialTheme.colorScheme.outline else MaterialTheme.colorScheme.onSurface,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

private fun tagSuggestions(typed: String, tags: List<TagNameModel>, allTags: List<TagNameModel>): List<String> {
    val prefix = typed.trim().lowercase(Locale.ROOT)
    if (prefix.isEmpty()) {
        return emptyList()
    }
    return allTags
        .filter { it !in tags && it.value.lowercase(Locale.ROOT).startsWith(prefix = prefix) }
        .take(n = TAG_SUGGESTIONS)
        .map { it.value }
}
