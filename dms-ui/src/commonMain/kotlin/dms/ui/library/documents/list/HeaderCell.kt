package dms.ui.library.documents.list

import androidx.compose.foundation.clickable
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextAlign
import dms.ui.library.documents.model.DocumentSortModel
import dms.ui.library.documents.model.SortColumn

/**
 * One column title of the document list. A sortable one ([column] set) sorts by itself on a click and
 * shows an arrow while the list is sorted by it.
 */
@Composable
fun HeaderCell(
    text: String,
    modifier: Modifier,
    column: SortColumn? = null,
    sort: DocumentSortModel? = null,
    onSort: (SortColumn) -> Unit = {},
    textAlign: TextAlign = TextAlign.Start,
) {
    val arrow = when {
        column == null || sort?.column != column -> ""
        sort.descending -> " ↓"
        else -> " ↑"
    }
    Text(
        text = text + arrow,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
        textAlign = textAlign,
        modifier = if (column == null) {
            modifier
        } else {
            modifier.clickable(role = Role.Button, onClickLabel = "Sort by $text") { onSort(column) }
        },
    )
}
