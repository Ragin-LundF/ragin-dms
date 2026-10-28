package dms.application.port.pdf

import dms.application.port.ai.model.PageImageModel
import dms.application.port.pdf.model.PageInput
import dms.application.port.pdf.model.PreparedPageModel
import dms.application.port.pdf.model.RebuiltFileModel
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rebuild.model.LayoutBoxModel

/**
 * Rebuilds a scanned PDF as a new PDF with real text where the scan could be read and verified, and the
 * original pixels everywhere else. The result waits in the app's own data, never in a library folder,
 * until the caller places or discards it; what is left over when the app ends is deleted at its next
 * start. The source never changes.
 */
interface PdfRebuilder {

    /** Page [page] of [path] rendered at full resolution and straightened. */
    suspend fun preparePage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
    ): StorageResult<PreparedPageModel>

    /** The part of [prepared] around [box], for reading one line again. */
    suspend fun crop(prepared: PreparedPageModel, box: LayoutBoxModel): PageImageModel

    /** Writes the rebuilt PDF of [source], whose content is [sourceHash], from [pages], in their order. */
    suspend fun write(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        sourceHash: ContentHashModel,
        pages: List<PageInput>,
    ): StorageResult<RebuiltFileModel>

    /** One page of [rebuilt] as a picture, or `null` if it cannot be rendered. */
    suspend fun renderPage(rebuilt: RebuiltFileModel, page: Int, maxSide: Int): PageImageModel?

    /** Copies [rebuilt] to [target] in [folder], which must not exist yet. */
    suspend fun place(
        rebuilt: RebuiltFileModel,
        folder: LibraryFolderModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /** Deletes [rebuilt]; it was never part of the library. */
    suspend fun discard(rebuilt: RebuiltFileModel)
}
