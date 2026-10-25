package dms.ui.library.documents.list

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
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
import androidx.compose.ui.text.style.TextAlign
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.library.DocumentColumns
import dms.ui.library.documents.GroupHeader
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.documents.model.DocumentGroupModel
import dms.ui.library.documents.model.DocumentSortModel
import dms.ui.library.documents.model.SortColumn
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableDocument
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The documents as rows under a header that stays in place while the rows scroll, grouped like the
 * grid; a click on a column title sorts by it. Each row can be dragged onto a folder.
 */
@Composable
fun DocumentList(
    groups: List<DocumentGroupModel>,
    selected: Set<DocumentIdModel>,
    inspected: DocumentIdModel?,
    sort: DocumentSortModel?,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    drag: DocumentDrag,
    onClick: (DocumentIdModel, DocumentClick) -> Unit,
    onToggle: (DocumentIdModel) -> Unit,
    onSort: (SortColumn) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        ListHeader(sort = sort, onSort = onSort)
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        LazyColumn(
            contentPadding = PaddingValues(bottom = Spacing.xxlarge * 3),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
        ) {
            groups.forEach { group ->
                stickyHeader(key = "group:${group.title}") {
                    GroupHeader(title = group.title, count = group.rows.size)
                }
                items(items = group.rows, key = { it.document.id.value }) { row ->
                    val id = row.document.id
                    DraggableDocument(drag = drag, document = row.document) {
                        DocumentListRow(
                            row = row,
                            selected = id in selected,
                            inspected = id == inspected,
                            selecting = selected.isNotEmpty(),
                            folderId = folderId,
                            thumbnails = thumbnails,
                            onClick = { click -> onClick(id, click) },
                            onToggle = { onToggle(id) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ListHeader(sort: DocumentSortModel?, onSort: (SortColumn) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = DocumentColumns.padding, vertical = Spacing.small),
        horizontalArrangement = Arrangement.spacedBy(space = DocumentColumns.gap),
    ) {
        Spacer(modifier = Modifier.width(width = LibrarySizes.selectionCircle))
        Spacer(modifier = Modifier.width(width = DocumentColumns.thumbnailWidth))
        HeaderCell(
            text = "Name",
            modifier = Modifier.weight(weight = DocumentColumns.NAME),
            column = SortColumn.NAME,
            sort = sort,
            onSort = onSort,
        )
        HeaderCell(
            text = "Category",
            modifier = Modifier.weight(weight = DocumentColumns.CATEGORY),
            column = SortColumn.CATEGORY,
            sort = sort,
            onSort = onSort,
        )
        HeaderCell(text = "Tags", modifier = Modifier.weight(weight = DocumentColumns.TAGS))
        HeaderCell(
            text = "From",
            modifier = Modifier.weight(weight = DocumentColumns.CORRESPONDENT),
            column = SortColumn.CORRESPONDENT,
            sort = sort,
            onSort = onSort,
        )
        HeaderCell(
            text = "Date",
            modifier = Modifier.weight(weight = DocumentColumns.DATE),
            column = SortColumn.DATE,
            sort = sort,
            onSort = onSort,
        )
        HeaderCell(
            text = "Amount",
            modifier = Modifier.weight(weight = DocumentColumns.AMOUNT),
            column = SortColumn.AMOUNT,
            sort = sort,
            onSort = onSort,
            textAlign = TextAlign.End,
        )
    }
}
