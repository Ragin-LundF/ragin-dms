package dms.ui.document.details

import androidx.compose.foundation.clickable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Rule
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
import dms.domain.retention.RetentionDates
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButtonModel
import dms.ui.document.details.model.LinkCandidateModel

/** The inspector's sections for fields, note, linked documents and retention. */
@Composable
fun DetailsSections(
    document: DocumentModel,
    catalog: CatalogModel,
    busyReason: String?,
    controller: DetailsController,
    onOpenDocument: (DocumentIdModel) -> Unit,
) {
    SectionHeader(title = "Fields")
    FieldsEditor(
        document = document,
        definitions = catalog.settings.fieldDefinitions,
        busyReason = busyReason,
        onSave = { controller.saveFields(document = document, typed = it) },
        onAddStandardFields = controller::addStandardFields,
    )
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
    RuleSection(document = document, busyReason = busyReason, controller = controller)
    SectionHeader(title = "Retention")
    val category = document.categoryId?.let { catalog.categories[it] }
    RetentionEditor(
        document = document,
        category = category,
        dueDate = RetentionDates.dueDate(document = document, catalog = catalog),
        busyReason = busyReason,
        onKeepUntil = { controller.saveKeepUntil(document = document, typed = it) },
        onCategoryYears = { typed ->
            category?.let { controller.saveCategoryYears(categoryId = it.id, typed = typed) }
        },
    )
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

@Composable
private fun RuleSection(document: DocumentModel, busyReason: String?, controller: DetailsController) {
    SectionHeader(title = "Rule")
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Create rule from this document",
                icon = Icons.Rounded.Rule,
                disabledReason = "Give the document a correspondent first"
                    .takeIf { document.metadata.correspondent == null },
            ) { controller.createRule(document = document) },
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
