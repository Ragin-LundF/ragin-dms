package dms.pdf

import dms.application.port.ai.model.PageImageModel
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.pdf.model.PdfMetadataModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.pdf.io.PdfFiles
import dms.pdf.io.PdfLocations
import dms.pdf.meta.PdfInfo
import dms.pdf.page.PageImages
import dms.pdf.page.PageRewriter
import dms.pdf.text.TextLayer
import java.nio.file.Path

/** The PDF editor on PDFBox. Sources are only read; every result is a new file. */
class PdfBoxEditor(private val locations: PdfLocations) : PdfEditor {

    override suspend fun pageCount(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Int> {
        return withSource(folder = folder, path = path) { file ->
            PdfFiles.load(file = file).use { StorageResult.Success(value = it.numberOfPages) }
        }
    }

    override suspend fun renderPage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
        maxSide: Int,
    ): PageImageModel? {
        val rendered = withSource(folder = folder, path = path) { file ->
            StorageResult.Success(value = PageImages.render(file = file, page = page, maxSide = maxSide))
        }
        return (rendered as? StorageResult.Success)?.value
    }

    override suspend fun rewrite(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<PageChangeModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return withTarget(folder = folder, source = source, target = target) { input, output ->
            PageRewriter.rewrite(source = input, pages = pages, target = output)
        }
    }

    override suspend fun merge(
        folder: LibraryFolderModel,
        sources: List<RelativePathModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return PdfFiles.io {
            val inputs = sources.map { source ->
                locations.existing(folder, source) ?: return@io notFound()
            }
            val output = locations.target(folder, target) ?: return@io unreachable()
            PageRewriter.merge(sources = inputs, target = output)
        }
    }

    override suspend fun hasTextLayer(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Boolean> {
        return withSource(folder = folder, path = path) { file ->
            StorageResult.Success(value = TextLayer.hasText(file = file))
        }
    }

    override suspend fun addTextLayer(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<String>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return withTarget(folder = folder, source = source, target = target) { input, output ->
            TextLayer.add(source = input, pages = pages, target = output)
        }
    }

    override suspend fun writeMetadata(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        metadata: PdfMetadataModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return withTarget(folder = folder, source = source, target = target) { input, output ->
            PdfInfo.write(source = input, metadata = metadata, target = output)
        }
    }

    override suspend fun readMetadata(
        folder: LibraryFolderModel,
        path: RelativePathModel,
    ): StorageResult<PdfMetadataModel> {
        return withSource(folder = folder, path = path) { file ->
            StorageResult.Success(value = PdfInfo.read(file = file))
        }
    }

    private suspend fun <T> withSource(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        block: (Path) -> StorageResult<T>,
    ): StorageResult<T> {
        return PdfFiles.io {
            val file = locations.existing(folder, path) ?: return@io notFound()
            block(file)
        }
    }

    private suspend fun withTarget(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        target: RelativePathModel,
        block: (Path, Path) -> StorageResult<FileFactsModel>,
    ): StorageResult<FileFactsModel> {
        return PdfFiles.io {
            val input = locations.existing(folder, source) ?: return@io notFound()
            val output = locations.target(folder, target) ?: return@io unreachable()
            block(input, output)
        }
    }

    private fun notFound(): StorageResult.Failure {
        return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
    }

    private fun unreachable(): StorageResult.Failure {
        return StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
    }
}
