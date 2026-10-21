package dms.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import dms.domain.document.DocumentId
import dms.ui.library.model.DocumentRow
import dms.ui.theme.Spacing

/** The documents the sidebar selects: name, category, tags, date and correspondent. */
@Composable
fun DocumentList(
    rows: List<DocumentRow>,
    inspected: DocumentId?,
    onInspect: (DocumentId) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(horizontal = Spacing.medium)) {
        Row(modifier = Modifier.fillMaxWidth().padding(vertical = Spacing.small)) {
            val style = MaterialTheme.typography.labelMedium
            ListCell(text = "Document (${rows.size})", weight = 3f, style = style)
            ListCell(text = "Category", weight = 1.5f, style = style)
            ListCell(text = "Tags", weight = 1.5f, style = style)
            ListCell(text = "Date", weight = 1f, style = style)
            ListCell(text = "Correspondent", weight = 1.5f, style = style)
        }
        HorizontalDivider()
        LazyColumn {
            items(items = rows, key = { it.document.id.value }) { row ->
                val document = row.document
                val background = if (document.id == inspected) {
                    MaterialTheme.colorScheme.secondaryContainer
                } else {
                    Color.Transparent
                }
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(color = background)
                        .clickable { onInspect(document.id) }
                        .padding(vertical = Spacing.small),
                ) {
                    val style = MaterialTheme.typography.bodyMedium
                    ListCell(text = document.metadata.title ?: document.path.fileName, weight = 3f, style = style)
                    ListCell(text = row.categoryName.orEmpty(), weight = 1.5f, style = style)
                    ListCell(text = document.tags.sorted().joinToString { it.value }, weight = 1.5f, style = style)
                    ListCell(text = document.metadata.documentDate?.toString().orEmpty(), weight = 1f, style = style)
                    ListCell(text = document.metadata.correspondent.orEmpty(), weight = 1.5f, style = style)
                }
            }
        }
    }
}
