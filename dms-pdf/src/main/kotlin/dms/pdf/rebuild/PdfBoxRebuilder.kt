package dms.pdf.rebuild

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
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.RebuiltPageModel
import dms.pdf.io.PdfFiles
import dms.pdf.io.PdfLocations
import dms.pdf.page.PageImages
import dms.pdf.rebuild.compose.PageComposer
import dms.pdf.rebuild.compose.PageWriter
import dms.pdf.rebuild.font.FontSet
import dms.pdf.rebuild.image.Deskew
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.rendering.ImageType
import org.apache.pdfbox.rendering.PDFRenderer
import java.awt.Rectangle
import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path
import java.util.UUID
import kotlin.io.path.deleteIfExists
import kotlin.io.path.listDirectoryEntries

/**
 * The rebuild on PDFBox. Rebuilt PDFs wait in [workDirectory], in the app's own data, until they are
 * placed in a library folder or discarded. The adapter is made once when the app starts, which is when
 * leftovers of an earlier run are deleted.
 */
class PdfBoxRebuilder(
    private val locations: PdfLocations,
    private val workDirectory: Path,
) : PdfRebuilder {

    init {
        discardAll()
    }

    override suspend fun preparePage(
        folder: LibraryFolderModel,
        path: RelativePathModel,
        page: Int,
    ): StorageResult<PreparedPageModel> {
        return PdfFiles.io {
            val file = locations.existing(folder, path) ?: return@io notFound()
            val rendered = PdfFiles.load(file = file).use { pdf ->
                if (page !in 0 until pdf.numberOfPages) return@io notFound()
                PDFRenderer(pdf).renderImageWithDPI(page, PageWriter.DPI, ImageType.RGB)
            }
            val angle = Deskew.angleOf(page = rendered)
            val straight = if (angle == 0.0) rendered else Deskew.rotate(page = rendered, angle = angle)
            val prepared = PreparedPageModel(pageIndex = page, image = Png.encode(image = straight), angle = angle)
            StorageResult.Success(value = prepared)
        }
    }

    override suspend fun crop(prepared: PreparedPageModel, box: LayoutBoxModel): PageImageModel {
        return withContext(context = Dispatchers.IO) {
            val page = Png.decode(image = prepared.image)
            val line = PixelBoxes(width = page.width, height = page.height).toPixels(box = box)
            val pad = maxOf(MIN_CROP_PAD, line.height * CROP_PAD_TENTHS / 10)
            val area = Rectangle(line.x - pad, line.y - pad, line.width + 2 * pad, line.height + 2 * pad)
                .intersection(Rectangle(0, 0, page.width, page.height))
            val copy = BufferedImage(area.width, area.height, BufferedImage.TYPE_INT_RGB)
            val graphics = copy.createGraphics()
            try {
                graphics.drawImage(page.getSubimage(area.x, area.y, area.width, area.height), 0, 0, null)
            } finally {
                graphics.dispose()
            }
            Png.encode(image = copy)
        }
    }

    override suspend fun write(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        sourceHash: ContentHashModel,
        pages: List<PageInput>,
    ): StorageResult<RebuiltFileModel> {
        return PdfFiles.io {
            val input = locations.existing(folder, source) ?: return@io notFound()
            val id = UUID.randomUUID().toString()
            val (summaries, saved) = PdfFiles.load(file = input).use { original ->
                PDDocument().use { rebuilt ->
                    val fonts = FontSet(document = rebuilt)
                    val summaries = pages.map { page ->
                        addPage(original = original, rebuilt = rebuilt, fonts = fonts, page = page)
                    }
                    summaries to PdfFiles.save(document = rebuilt, target = fileOf(id = id))
                }
            }
            when (saved) {
                is StorageResult.Failure -> saved
                is StorageResult.Success -> StorageResult.Success(
                    value = RebuiltFileModel(
                        id = id,
                        pages = summaries,
                        size = saved.value.size,
                        sourceHash = sourceHash,
                    ),
                )
            }
        }
    }

    override suspend fun renderPage(rebuilt: RebuiltFileModel, page: Int, maxSide: Int): PageImageModel? {
        val rendered = PdfFiles.io {
            val image = PageImages.render(file = fileOf(id = rebuilt.id), page = page, maxSide = maxSide)
            StorageResult.Success(value = image)
        }
        return (rendered as? StorageResult.Success)?.value
    }

    override suspend fun place(
        rebuilt: RebuiltFileModel,
        folder: LibraryFolderModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return PdfFiles.io {
            val output = locations.target(folder, target)
                ?: return@io StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            Files.createDirectories(output.parent)
            val temp = PdfFiles.tempFor(target = output)
            try {
                Files.copy(fileOf(id = rebuilt.id), temp)
                PdfFiles.place(temp = temp, target = output)
            } finally {
                Files.deleteIfExists(temp)
            }
        }
    }

    override suspend fun discard(rebuilt: RebuiltFileModel) {
        withContext(context = Dispatchers.IO) {
            fileOf(id = rebuilt.id).deleteIfExists()
        }
    }

    /** Rebuilds left over from an earlier run were never chosen; they are deleted when the app starts. */
    private fun discardAll() {
        // Cleaning up never keeps the app from starting; what cannot be deleted waits for the next start.
        val leftovers = runCatching { workDirectory.listDirectoryEntries() }.getOrDefault(defaultValue = emptyList())
        leftovers.forEach { runCatching { it.deleteIfExists() } }
    }

    private fun addPage(original: PDDocument, rebuilt: PDDocument, fonts: FontSet, page: PageInput): RebuiltPageModel {
        return when (page) {
            is PageInput.Failed -> {
                rebuilt.importPage(original.getPage(page.pageIndex))
                PageComposer.failed()
            }
            is PageInput.Read -> PageComposer.add(
                document = rebuilt,
                fonts = fonts,
                page = Png.decode(image = page.prepared.image),
                angle = page.prepared.angle,
                layout = page.layout,
            )
        }
    }

    /** The file of a rebuilt PDF; [id] must be one of ours, so it can never point outside [workDirectory]. */
    private fun fileOf(id: String): Path {
        return workDirectory.resolve(UUID.fromString(id).toString() + ".pdf")
    }

    private fun notFound(): StorageResult.Failure {
        return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
    }

    private companion object {
        const val MIN_CROP_PAD = 6
        const val CROP_PAD_TENTHS = 4
    }
}
