package dms.application.pdf.rebuild

import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.pdf.rebuild.model.RebuildBlocker
import dms.application.pdf.rebuild.model.RebuildOutcome
import dms.application.port.ai.PageLayoutReader
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.PdfRebuilder
import dms.application.port.pdf.model.PageInput
import dms.application.port.pdf.model.PreparedPageModel
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel

/**
 * "Convert to real PDF": rebuilds a scanned PDF page by page with the AI the user configured. The result
 * waits outside the library for the user's choice ([KeepRebuild]); nothing in the library changes here.
 * A page the AI cannot read in time stays as it was; an AI that cannot be reached or cannot read images
 * stops the run. [onPage] hears before each page starts, and with `page == pages` before the file is
 * written. A PDF that changed during the run is not written, since its pages would come from two
 * versions. Cancelling the caller cancels the run, and nothing is left behind, since the file is only
 * written at the end.
 */
class RebuildPdf(
    private val store: CatalogStore,
    private val pdf: PdfEditor,
    private val rebuilder: PdfRebuilder,
    private val reader: PageLayoutReader,
    private val availability: RebuildAvailability,
) {

    suspend operator fun invoke(
        documentId: DocumentIdModel,
        onPage: suspend (page: Int, pages: Int) -> Unit = { _, _ -> },
    ): RebuildOutcome {
        val library = store.current() ?: return RebuildOutcome.Failed(outcome = OperationOutcome.LibraryNotOpen)
        val document = library.catalog.documents[documentId]
            ?: return RebuildOutcome.Failed(outcome = OperationOutcome.NotFound)
        availability.blockerOf(library = library, document = document)
            ?.let { return RebuildOutcome.NotAvailable(reason = it) }
        val primary = library.folders.primary
        val pages = when (val counted = pdf.pageCount(folder = primary, path = document.path)) {
            is StorageResult.Failure -> return failed(failure = counted)
            is StorageResult.Success -> counted.value
        }
        val inputs = mutableListOf<PageInput>()
        for (page in 0 until pages) {
            onPage(page, pages)
            when (val read = readPage(library = library, document = document, page = page)) {
                is PageRead.Input -> inputs += read.input
                is PageRead.Stop -> return read.outcome
            }
        }
        onPage(pages, pages)
        return write(document = document, inputs = inputs)
    }

    /** Writes the rebuild unless the document changed during the run. */
    private suspend fun write(document: DocumentModel, inputs: List<PageInput>): RebuildOutcome {
        val library = store.current() ?: return RebuildOutcome.Failed(outcome = OperationOutcome.LibraryNotOpen)
        val current = library.catalog.documents[document.id]
            ?: return RebuildOutcome.Failed(outcome = OperationOutcome.NotFound)
        if (current.contentHash != document.contentHash) {
            return RebuildOutcome.NotAvailable(reason = RebuildBlocker.SOURCE_CHANGED)
        }
        val written = rebuilder.write(
            folder = library.folders.primary,
            source = current.path,
            sourceHash = document.contentHash,
            pages = inputs,
        )
        return when (written) {
            is StorageResult.Failure -> failed(failure = written)
            is StorageResult.Success -> RebuildOutcome.Rebuilt(rebuilt = written.value, originalSize = document.size)
        }
    }

    private suspend fun readPage(library: LibrarySnapshotModel, document: DocumentModel, page: Int): PageRead {
        val result = rebuilder.preparePage(folder = library.folders.primary, path = document.path, page = page)
        val prepared = when (result) {
            is StorageResult.Failure -> return PageRead.Stop(outcome = failed(failure = result))
            is StorageResult.Success -> result.value
        }
        return when (val layout = reader.readLayout(page = prepared.image)) {
            is LayoutOutcome.Read -> {
                val repaired = repaired(layout = layout.layout, prepared = prepared)
                PageRead.Input(input = PageInput.Read(prepared = prepared, layout = repaired))
            }
            is LayoutOutcome.Failure -> blockerOf(reason = layout.reason)
                ?.let { PageRead.Stop(outcome = RebuildOutcome.NotAvailable(reason = it)) }
                ?: PageRead.Input(input = PageInput.Failed(pageIndex = page))
        }
    }

    /** Lines with lost characters, read again from a crop; a line that does not fit stays an image. */
    private suspend fun repaired(layout: PageLayoutModel, prepared: PreparedPageModel): PageLayoutModel {
        val lines = layout.lines.map { line ->
            if (LineRepair.hasLostCharacters(text = line.text)) repairLine(line = line, prepared = prepared) else line
        }
        return layout.copy(lines = lines)
    }

    private suspend fun repairLine(line: LayoutLineModel, prepared: PreparedPageModel): LayoutLineModel {
        val crop = rebuilder.crop(prepared = prepared, box = line.box)
        val reread = reader.readLine(line = crop) as? LineOutcome.Read
        val text = reread?.let { LineRepair.repaired(text = line.text, reread = it.text) }
            ?: return line.copy(kept = KeepReason.LOST_CHARACTERS)
        return line.copy(text = text)
    }

    /** An AI that cannot be reached or cannot read images will not read the next page either. */
    private fun blockerOf(reason: ClassificationFailure): RebuildBlocker? {
        return when (reason) {
            ClassificationFailure.PROVIDER_UNAVAILABLE -> RebuildBlocker.AI_NOT_REACHABLE
            ClassificationFailure.MODEL_CANNOT_READ_IMAGES -> RebuildBlocker.MODEL_CANNOT_READ_IMAGES
            ClassificationFailure.TIMEOUT, ClassificationFailure.INVALID_RESPONSE -> null
        }
    }

    private fun failed(failure: StorageResult.Failure): RebuildOutcome {
        return RebuildOutcome.Failed(outcome = OperationOutcome.StorageFailed(failure = failure))
    }

    private sealed interface PageRead {
        data class Input(val input: PageInput) : PageRead
        data class Stop(val outcome: RebuildOutcome) : PageRead
    }
}
