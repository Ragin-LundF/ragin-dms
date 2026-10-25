package dms.ui.settings.library

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Rule
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle

/**
 * The matching rules: what each one looks for and proposes, whether it was learned or written by
 * hand, on or off. New rules come from accepted proposals on the dashboard and from "Create rule from
 * this document" in the quickLookViewModel.
 */
@Composable
fun RulesCard(
    settings: CatalogSettingsModel,
    categoryName: (CategoryIdModel) -> String,
    busyReason: String?,
    controller: LibrarySettingsController,
) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Matching rules",
            icon = Icons.Rounded.Rule,
            tint = MaterialTheme.colorScheme.tertiary,
            subtitle = "Values proposed without the AI; you still approve every suggestion",
        )
        if (settings.rules.isEmpty()) {
            Text(
                text = "No rules yet. Accept a learned rule on the dashboard, or create one from a document in the " +
                    "quickLookViewModel.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        settings.rules.forEach { rule ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(weight = 1f)) {
                    val origin = if (rule.origin == RuleOrigin.LEARNED) "learned" else "manual"
                    Text(text = "${rule.name} ($origin)", style = MaterialTheme.typography.titleSmall)
                    Text(
                        text = describe(rule = rule, categoryName = categoryName),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Switch(
                    checked = rule.enabled,
                    onCheckedChange = { controller.setRuleEnabled(id = rule.id, enabled = it) },
                    enabled = busyReason == null,
                )
                IconButton(onClick = { controller.removeRule(id = rule.id) }, enabled = busyReason == null) {
                    Icon(imageVector = Icons.Rounded.Delete, contentDescription = "Remove ${rule.name}")
                }
            }
        }
    }
}

private fun describe(rule: MatchingRuleModel, categoryName: (CategoryIdModel) -> String): String {
    val conditions = rule.conditions.joinToString(separator = " and ") { condition ->
        when (condition) {
            is RuleCondition.CorrespondentIs -> "from ${condition.name}"
            is RuleCondition.TextContains ->
                "text has ${if (condition.requireAll) "all of" else "any of"} ${condition.words.joinToString()}"

            is RuleCondition.FileNameMatches -> "name like ${condition.glob}"
        }
    }
    val actions = listOfNotNull(
        rule.actions.categoryId?.let { "category ${categoryName(it)}" },
        rule.actions.tags.takeIf { it.isNotEmpty() }?.let { tags -> "tags ${tags.joinToString { it.value }}" },
        rule.actions.fields.takeIf { it.isNotEmpty() }?.let { "${it.size} field values" },
        rule.actions.targetFolder?.let { "folder ${it.value}" },
    )
    return "When $conditions: ${actions.joinToString(separator = ", ")}"
}
