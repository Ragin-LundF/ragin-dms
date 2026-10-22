package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.document.model.DetailEdit
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.link.model.DocumentLinkModel

/** Applies one change to a document's fields, note, retention or links, as one catalog change. */
class EditDocumentDetails(
    private val store: CatalogStore,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(edit: DetailEdit): OperationOutcome {
        val outcome = store.commit { catalog -> apply(catalog = catalog, edit = edit) }
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.CATALOG_EDITED, subject = edit::class.simpleName.orEmpty())
        }
        return outcome
    }

    private fun apply(catalog: CatalogModel, edit: DetailEdit): CatalogUpdate {
        return when (edit) {
            is DetailEdit.SetFields -> catalog.setFields(documentId = edit.documentId, fields = edit.fields)
            is DetailEdit.SetNote -> catalog.setNote(documentId = edit.documentId, note = edit.note)
            is DetailEdit.SetRetention -> catalog.setRetention(documentId = edit.documentId, retention = edit.retention)
            is DetailEdit.SetRetentionYears -> catalog.setRetentionYears(
                categoryId = edit.categoryId,
                years = edit.years,
            )
            is DetailEdit.Link -> catalog.link(
                link = DocumentLinkModel.of(first = edit.first, second = edit.second, label = edit.label),
            )
            is DetailEdit.Unlink -> catalog.unlink(first = edit.first, second = edit.second)
        }
    }
}
