package dms.application.fake.pdf

import dms.application.fake.storage.FakeFolders
import dms.application.port.ai.model.PageImageModel
import dms.application.port.pdf.PdfEditor
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.pdf.model.PdfMetadataModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/**
 * PDFs as text in [FakeFolders]: pages separated by `|`, a page's rotation as `@90`, its text layer as
 * `#text`, and embedded metadata as a trailing `;title`. Enough to see what an operation did.
 */
class FakePdfEditor(private val folders: FakeFolders) : PdfEditor {

    override suspend fun pageCount(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Int> {
        val content = folders.content(folder = folder, path = path.value) ?: return missing()
        return StorageResult.Success(value = pagesOf(content = content).size)
    }

    override suspend fun renderPage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
        maxSide: Int,
    ): PageImageModel? {
        return null
    }

    override suspend fun rewrite(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<PageChangeModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val content = folders.content(folder = folder, path = source.value) ?: return missing()
        val existing = pagesOf(content = content)
        val rewritten = pages.joinToString(separator = "|") { change ->
            val page = existing[change.page]
            if (change.rotation == 0) page else "$page@${change.rotation}"
        }
        return write(folder = folder, target = target, content = rewritten)
    }

    override suspend fun merge(
        folder: LibraryFolderModel,
        sources: List<RelativePathModel>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val contents = sources.map { folders.content(folder = folder, path = it.value) ?: return missing() }
        return write(folder = folder, target = target, content = contents.joinToString(separator = "|"))
    }

    override suspend fun hasTextLayer(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Boolean> {
        val content = folders.content(folder = folder, path = path.value) ?: return missing()
        return StorageResult.Success(value = pagesOf(content = content).all { it.contains(char = '#') })
    }

    override suspend fun addTextLayer(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        pages: List<String>,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val content = folders.content(folder = folder, path = source.value) ?: return missing()
        val layered = pagesOf(content = content).mapIndexed { index, page ->
            "$page#${pages.getOrElse(index = index) { "" }}"
        }
        return write(folder = folder, target = target, content = layered.joinToString(separator = "|"))
    }

    override suspend fun writeMetadata(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        metadata: PdfMetadataModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val content = folders.content(folder = folder, path = source.value) ?: return missing()
        return write(
            folder = folder,
            target = target,
            content = "${content.substringBefore(delimiter = ';')};${metadata.title}",
        )
    }

    override suspend fun readMetadata(
        folder: LibraryFolderModel,
        path: RelativePathModel,
    ): StorageResult<PdfMetadataModel> {
        val content = folders.content(folder = folder, path = path.value) ?: return missing()
        return StorageResult.Success(
            value = PdfMetadataModel(
                title = content.substringAfter(delimiter = ';', missingDelimiterValue = "").ifEmpty { null },
            ),
        )
    }

    private fun pagesOf(content: String): List<String> {
        return content.substringBefore(delimiter = ';').split('|')
    }

    private fun write(
        folder: LibraryFolderModel,
        target: RelativePathModel,
        content: String,
    ): StorageResult<FileFactsModel> {
        if (folders.content(folder = folder, path = target.value) != null) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        folders.put(folder = folder, path = target.value, content = content)
        return StorageResult.Success(value = FakeFolders.factsOf(content = content))
    }

    private fun missing(): StorageResult.Failure {
        return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
    }
}
