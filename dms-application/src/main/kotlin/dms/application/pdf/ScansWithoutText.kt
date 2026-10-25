package dms.application.pdf

import dms.application.library.CatalogStore
import dms.application.pdf.model.ScansOutcome
import dms.application.port.pdf.PdfEditor
import dms.application.port.storage.model.StorageResult
import dms.domain.document.DocumentKind
import dms.domain.document.model.ContentHashModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * The PDFs without a text layer, such as scans without OCR. Each file is checked once and the answer
 * kept until it changes. Only reads.
 */
class ScansWithoutText(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
) {

    private val mutex = Mutex()
    private val known = HashMap<ContentHashModel, Boolean>()

    suspend operator fun invoke(): ScansOutcome {
        val library = store.current() ?: return ScansOutcome.LibraryNotOpen
        val scans = mutex.withLock {
            library.catalog.documents.values
                .filter { it.kind == DocumentKind.PDF }
                .filter { document ->
                    val hasText = known.getOrPut(key = document.contentHash) {
                        val checked = pdf.hasTextLayer(folder = library.folders.primary, path = document.path)
                        // A file that cannot be read is not offered; making it searchable would fail too.
                        (checked as? StorageResult.Success)?.value ?: true
                    }
                    !hasText
                }
                .sortedBy { it.path.value }
                .map { it.id }
        }
        return ScansOutcome.Found(documentIds = scans)
    }
}
