package dms.ui.library.documents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.document.DocumentId
import dms.ui.library.DocumentColumns
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableDocument
import dms.ui.library.model.DocumentRow
import dms.ui.theme.Spacing

/** The documents as rows under a column header; each row can be dragged onto the sidebar. */
@Composable
fun DocumentList(
    rows: List<DocumentRow>,
    inspected: DocumentId?,
    drag: DocumentDrag,
    onInspect: (DocumentId) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = DocumentColumns.padding, vertical = Spacing.small),
            horizontalArrangement = Arrangement.spacedBy(space = DocumentColumns.gap),
        ) {
            Spacer(modifier = Modifier.width(width = DocumentColumns.badge))
            HeaderCell(text = "Name", modifier = Modifier.weight(weight = DocumentColumns.NAME))
            HeaderCell(text = "Category", modifier = Modifier.weight(weight = DocumentColumns.CATEGORY))
            HeaderCell(text = "Tags", modifier = Modifier.weight(weight = DocumentColumns.TAGS))
            HeaderCell(text = "Date", modifier = Modifier.weight(weight = DocumentColumns.DATE))
            HeaderCell(text = "Size", modifier = Modifier.weight(weight = DocumentColumns.SIZE))
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        LazyColumn(
            modifier = Modifier.padding(top = Spacing.xsmall),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
        ) {
            items(items = rows, key = { it.document.id.value }) { row ->
                DraggableDocument(drag = drag, document = row.document) {
                    DocumentListRow(row = row, selected = row.document.id == inspected) {
                        onInspect(row.document.id)
                    }
                }
            }
        }
    }
}

