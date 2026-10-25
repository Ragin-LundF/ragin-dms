package dms.ui.library.documents.grid

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.library.documents.GroupHeader
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.documents.model.DocumentGroupModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableDocument
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The documents as a grid of page previews, as many columns of at least [LibrarySizes.cardMinWidth] as
 * fit, under the sticky header of each group unless [headers] is off. Each card can be dragged onto a folder.
 */
@Composable
fun DocumentGrid(
    groups: List<DocumentGroupModel>,
    selected: Set<DocumentIdModel>,
    inspected: DocumentIdModel?,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    drag: DocumentDrag,
    onClick: (DocumentIdModel, DocumentClick) -> Unit,
    onToggle: (DocumentIdModel) -> Unit,
    modifier: Modifier = Modifier,
    headers: Boolean = true,
) {
    val gap = Spacing.medium
    BoxWithConstraints(modifier = modifier) {
        val columns = ((maxWidth + gap) / (LibrarySizes.cardMinWidth + gap)).toInt()
            .coerceAtLeast(minimumValue = 1)
        LazyColumn(contentPadding = PaddingValues(bottom = Spacing.xxlarge * 3)) {
            groups.forEach { group ->
                if (headers) {
                    stickyHeader(key = "group:${group.title}") {
                        GroupHeader(title = group.title, count = group.rows.size)
                    }
                }
                items(
                    items = group.rows.chunked(size = columns),
                    key = { "line:${it.first().document.id.value}" },
                ) { line ->
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(bottom = gap),
                        horizontalArrangement = Arrangement.spacedBy(space = gap),
                    ) {
                        line.forEach { row ->
                            val id = row.document.id
                            DraggableDocument(
                                drag = drag,
                                document = row.document,
                                modifier = Modifier.weight(weight = 1f),
                            ) {
                                DocumentCard(
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
                        repeat(times = columns - line.size) { Spacer(modifier = Modifier.weight(weight = 1f)) }
                    }
                }
            }
        }
    }
}
