package dms.application.port.pdf

import dms.application.port.ai.model.PageImageModel
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.pdf.model.PdfMetadataModel
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/**
 * Reads and writes PDF files in one library folder. Every write creates a new file at a target that
 * must not exist yet (written to a temporary file and moved into place); the source never changes.
 * Replacing a document's file with the new version is the caller's job.
 */
interface PdfEditor {

    suspend fun pageCount(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Int>

    /** One page as a small picture for the page editor, or `null` if it cannot be rendered. */
    suspend fun renderPage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
        maxSide: Int,
    ): PageImageModel?

    /**
     * A new PDF of the [pages] of [source], in their order and turned as given; this rotates, deletes,
     * reorders and extracts.
     */
    suspend fun rewrite(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<PageChangeModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /** A new PDF of every page of [sources], one after the other. */
    suspend fun merge(
        folder: LibraryFolderModel,
        sources: List<RelativePathModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /** Whether the PDF has a usable text layer on its pages, as a scan without OCR has not. */
    suspend fun hasTextLayer(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Boolean>

    /** A copy of [source] with [pages] as an invisible text layer, one text per page, so it can be searched. */
    suspend fun addTextLayer(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<String>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /** A copy of [source] with [metadata] in its document information and XMP. */
    suspend fun writeMetadata(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        metadata: PdfMetadataModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    suspend fun readMetadata(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<PdfMetadataModel>
}
