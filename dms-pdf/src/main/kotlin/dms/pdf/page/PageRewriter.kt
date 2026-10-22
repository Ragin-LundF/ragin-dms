package dms.pdf.page

import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.pdf.io.PdfFiles
import org.apache.pdfbox.pdmodel.PDDocument
import java.nio.file.Path

/** New PDFs from the pages of existing ones: rotate, delete, reorder, extract and merge. */
object PageRewriter {

    private const val FULL_TURN = 360

    fun rewrite(source: Path, pages: List<PageChangeModel>, target: Path): StorageResult<FileFactsModel> {
        require(value = pages.isNotEmpty()) { "A PDF needs at least one page" }
        PdfFiles.load(file = source).use { input ->
            require(value = pages.all { it.page < input.numberOfPages }) {
                "The PDF has only ${input.numberOfPages} pages"
            }
            PDDocument().use { output ->
                pages.forEach { change ->
                    val page = input.getPage(change.page)
                    val imported = output.importPage(page)
                    imported.rotation = Math.floorMod(page.rotation + change.rotation, FULL_TURN)
                }
                return PdfFiles.save(document = output, target = target)
            }
        }
    }

    fun merge(sources: List<Path>, target: Path): StorageResult<FileFactsModel> {
        require(value = sources.isNotEmpty()) { "Nothing to merge" }
        val inputs = mutableListOf<PDDocument>()
        try {
            PDDocument().use { output ->
                sources.forEach { source ->
                    val input = PdfFiles.load(file = source)
                    inputs += input
                    input.pages.forEach { output.importPage(it) }
                }
                return PdfFiles.save(document = output, target = target)
            }
        } finally {
            inputs.forEach { it.close() }
        }
    }
}
