package dms.application.pdf

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.pdf.model.SearchableOutcome
import dms.application.port.pdf.OcrTool
import dms.application.port.pdf.PdfEditor
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.StorageResult
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel

/**
 * Makes a scanned PDF searchable: a new version with an invisible text layer from the transcription
 * the AI made of it (one text per page), or, with [precise], from the OCR program installed on this
 * device. PDFs that already have a text layer are left as they are.
 */
class MakeSearchable(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
    private val ocr: OcrTool,
    private val texts: TextStore,
    private val versions: PdfVersions,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(documentId: DocumentIdModel, precise: Boolean = false): SearchableOutcome {
        val library = store.current() ?: return SearchableOutcome.Failed(outcome = OperationOutcome.LibraryNotOpen)
        val document = library.catalog.documents[documentId]
            ?: return SearchableOutcome.Failed(outcome = OperationOutcome.NotFound)
        if (document.kind != DocumentKind.PDF) {
            return SearchableOutcome.NotPdf
        }
        when (val hasText = pdf.hasTextLayer(folder = library.folders.primary, path = document.path)) {
            is StorageResult.Failure -> return SearchableOutcome.Failed(
                outcome = OperationOutcome.StorageFailed(failure = hasText),
            )
            is StorageResult.Success -> if (hasText.value) return SearchableOutcome.AlreadySearchable
        }
        val outcome = when (val written = newVersion(library = library, document = document, precise = precise)) {
            is VersionResult.Missing -> return written.outcome
            is VersionResult.Written -> written.outcome
        }
        if (outcome !is OperationOutcome.Done) {
            return SearchableOutcome.Failed(outcome = outcome)
        }
        activity.record(kind = ActivityKind.PDF_EDITED, subject = "Searchable: ${document.path}")
        return SearchableOutcome.Made(outcome = outcome)
    }

    /** The searchable version from the OCR program or from the transcription, or why there is none. */
    private suspend fun newVersion(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        precise: Boolean,
    ): VersionResult {
        if (precise) {
            if (!ocr.available()) {
                return VersionResult.Missing(outcome = SearchableOutcome.OcrUnavailable)
            }
            val outcome = versions.replace(library = library, document = document) { folder, target ->
                ocr.ocr(folder = folder, source = document.path, target = target)
            }
            return VersionResult.Written(outcome = outcome)
        }
        val transcription = texts.read(folder = library.folders.primary, documentId = document.id)
            ?.takeIf { it.isNotBlank() }
            ?: return VersionResult.Missing(outcome = SearchableOutcome.NoTranscription)
        val outcome = versions.replace(library = library, document = document) { folder, target ->
            pdf.addTextLayer(
                folder = folder,
                source = document.path,
                pages = transcription.split(PAGE_BREAK),
                target = target,
            )
        }
        return VersionResult.Written(outcome = outcome)
    }

    private sealed interface VersionResult {
        data class Written(val outcome: OperationOutcome) : VersionResult
        data class Missing(val outcome: SearchableOutcome) : VersionResult
    }

    companion object {
        /** Separates the pages of a transcription in the text store. */
        const val PAGE_BREAK: Char = '\u000C'
    }
}
