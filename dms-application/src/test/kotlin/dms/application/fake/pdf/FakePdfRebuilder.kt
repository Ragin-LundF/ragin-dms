package dms.application.fake.pdf

import dms.application.fake.storage.FakeFolders
import dms.application.port.ai.model.PageImageModel
import dms.application.port.pdf.PdfRebuilder
import dms.application.port.pdf.model.PageInput
import dms.application.port.pdf.model.PreparedPageModel
import dms.application.port.pdf.model.RebuiltFileModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.RebuiltPageModel

/**
 * Rebuilds as text, waiting outside the library in [waiting]: a read page becomes its line texts joined
 * by `/`, with `[image]` for a kept line, and a failed page becomes `page n kept`. A prepared page's image
 * is `page n`, a crop's image is its box, so a fake reader can tell them apart.
 */
class FakePdfRebuilder(private val folders: FakeFolders) : PdfRebuilder {

    val waiting: MutableMap<String, String> = mutableMapOf()
    private var next = 0

    override suspend fun preparePage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
    ): StorageResult<PreparedPageModel> {
        folders.content(folder = folder, path = path.value)
            ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        return StorageResult.Success(
            value = PreparedPageModel(pageIndex = page, image = imageOf(text = "page $page"), angle = 0.0),
        )
    }

    override suspend fun crop(prepared: PreparedPageModel, box: LayoutBoxModel): PageImageModel {
        return imageOf(text = box.toString())
    }

    override suspend fun write(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        sourceHash: ContentHashModel,
        pages: List<PageInput>,
    ): StorageResult<RebuiltFileModel> {
        val content = pages.joinToString(separator = "|") { page ->
            when (page) {
                is PageInput.Failed -> "page ${page.pageIndex} kept"
                is PageInput.Read -> page.layout.lines.joinToString(separator = "/") { line ->
                    if (line.kept == null) line.text else "[image]"
                }
            }
        }
        val id = "rebuild-${next++}"
        waiting[id] = content
        val summaries = pages.map { summaryOf(page = it) }
        val rebuilt = RebuiltFileModel(
            id = id,
            pages = summaries,
            size = content.length.toLong(),
            sourceHash = sourceHash,
        )
        return StorageResult.Success(value = rebuilt)
    }

    override suspend fun renderPage(rebuilt: RebuiltFileModel, page: Int, maxSide: Int): PageImageModel? {
        return null
    }

    override suspend fun place(
        rebuilt: RebuiltFileModel,
        folder: LibraryFolderModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val content = waiting[rebuilt.id] ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        if (folders.content(folder = folder, path = target.value) != null) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        folders.put(folder = folder, path = target.value, content = content)
        return StorageResult.Success(value = FakeFolders.factsOf(content = content))
    }

    override suspend fun discard(rebuilt: RebuiltFileModel) {
        waiting.remove(key = rebuilt.id)
    }

    private fun summaryOf(page: PageInput): RebuiltPageModel {
        val lines = (page as? PageInput.Read)?.layout?.lines.orEmpty()
        val failed = listOfNotNull(KeepReason.PAGE_FAILED.takeIf { page is PageInput.Failed })
        val kept = lines.mapNotNull { it.kept } + failed
        return RebuiltPageModel(
            linesAsText = lines.size - lines.count { it.kept != null },
            linesTotal = lines.size,
            angle = 0.0,
            keptReasons = kept.groupingBy { it }.eachCount(),
            regions = emptyList(),
        )
    }

    private fun imageOf(text: String): PageImageModel {
        return PageImageModel(bytes = text.encodeToByteArray(), mimeType = "image/png")
    }
}
