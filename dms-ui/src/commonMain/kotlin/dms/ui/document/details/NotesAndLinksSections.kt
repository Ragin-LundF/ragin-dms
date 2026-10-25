package dms.ui.document.details

import androidx.compose.foundation.clickable
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.ui.components.SectionHeader
import dms.ui.document.details.model.LinkCandidateModel

/** The inspector's sections for the note, the linked documents and the related ones. */
@Composable
fun NotesAndLinksSections(
    document: DocumentModel,
    catalog: CatalogModel,
    busyReason: String?,
    controller: DetailsController,
    onOpenDocument: (DocumentIdModel) -> Unit,
) {
    SectionHeader(title = "Note")
    NoteEditor(document = document, enabled = busyReason == null) { note ->
        controller.saveNote(document = document, note = note)
    }
    SectionHeader(title = "Linked documents")
    LinksEditor(
        links = controller.linksOf(document = document, catalog = catalog),
        busyReason = busyReason,
        onOpen = onOpenDocument,
        onUnlink = { controller.unlink(document = document, other = it) },
        onAdd = { controller.startLink(document = document) },
    )
    RelatedSection(document = document, controller = controller, onOpenDocument = onOpenDocument)
    val pick by controller.linkPick.collectAsState()
    pick?.let {
        LinkPickerDialog(
            pick = it,
            onSearch = controller::search,
            onLabel = controller::setLinkLabel,
            onPick = controller::link,
            onDismiss = controller::cancelLink,
        )
    }
}

@Composable
private fun RelatedSection(
    document: DocumentModel,
    controller: DetailsController,
    onOpenDocument: (DocumentIdModel) -> Unit,
) {
    val related by produceState(
        initialValue = emptyList<LinkCandidateModel>(),
        key1 = document.id,
        key2 = document.contentHash,
    ) {
        value = controller.related(document = document)
    }
    if (related.isNotEmpty()) {
        SectionHeader(title = "Related documents")
        related.forEach { candidate ->
            Text(
                text = candidate.name,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.primary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.clickable { onOpenDocument(candidate.documentId) },
            )
        }
    }
}
