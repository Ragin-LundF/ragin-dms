package dms.application.pdf

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.state.model.ActivityKind
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel

/**
 * Rotates, deletes and reorders the pages of a PDF document in place: [pages] lists the pages of the
 * new version in their order. The old version goes to the system trash in every folder.
 */
class EditPages(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
    private val versions: PdfVersions,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(documentId: DocumentIdModel, pages: List<PageChangeModel>): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        if (document.kind != DocumentKind.PDF || pages.isEmpty()) {
            return OperationOutcome.NotFound
        }
        val outcome = versions.replace(library = library, document = document) { folder, target ->
            pdf.rewrite(folder = folder, source = document.path, pages = pages, target = target)
        }
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.PDF_EDITED, subject = document.path.value)
        }
        return outcome
    }
}
