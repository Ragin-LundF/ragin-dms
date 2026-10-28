package dms.application.pdf.rebuild

import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.pdf.rebuild.model.RebuildBlocker
import dms.application.port.pdf.PdfEditor
import dms.application.port.storage.model.StorageResult
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel

/**
 * Whether "Convert to real PDF" is offered for a document: a PDF without a real text layer. Whether the
 * AI can read images shows only when it is asked, so the first run says so if it cannot.
 */
class RebuildAvailability(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
) {

    suspend operator fun invoke(documentId: DocumentIdModel): Boolean {
        val library = store.current() ?: return false
        val document = library.catalog.documents[documentId] ?: return false
        return blockerOf(library = library, document = document) == null
    }

    /** Why [document] cannot be rebuilt, or `null` if it can. A file that cannot be read is not offered. */
    suspend fun blockerOf(library: LibrarySnapshotModel, document: DocumentModel): RebuildBlocker? {
        if (document.kind != DocumentKind.PDF) {
            return RebuildBlocker.NOT_PDF
        }
        val hasText = pdf.hasTextLayer(folder = library.folders.primary, path = document.path)
        return RebuildBlocker.HAS_TEXT_LAYER.takeIf { (hasText as? StorageResult.Success)?.value ?: true }
    }
}
