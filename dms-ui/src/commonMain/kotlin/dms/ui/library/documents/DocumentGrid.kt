package dms.ui.library.documents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentIdModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableDocument
import dms.ui.library.model.DocumentRowModel
import dms.ui.theme.Spacing

/** The documents as a grid of cards; each card can be dragged onto the sidebar. */
@Composable
fun DocumentGrid(
    rows: List<DocumentRowModel>,
    inspected: DocumentIdModel?,
    drag: DocumentDrag,
    onInspect: (DocumentIdModel) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 220.dp),
        modifier = modifier,
        contentPadding = PaddingValues(vertical = Spacing.small),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        items(items = rows, key = { it.document.id.value }) { row ->
            DraggableDocument(drag = drag, document = row.document) {
                DocumentCard(row = row, selected = row.document.id == inspected) { onInspect(row.document.id) }
            }
        }
    }
}
