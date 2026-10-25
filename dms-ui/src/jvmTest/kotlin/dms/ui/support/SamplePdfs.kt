package dms.ui.support

import java.nio.file.Files
import java.nio.file.Path

/**
 * Writes tiny synthetic PDF files: [pages] landscape pages (twice as wide as high), each with the
 * text "Page n" in a standard font, so they have a text layer and a turn can be seen in a rendering.
 */
object SamplePdfs {

    private const val FIRST_PAGE_OBJECT = 4

    fun write(file: Path, pages: Int): Path {
        Files.createDirectories(file.parent)
        Files.write(file, bytesOf(pages = pages))
        return file
    }

    private fun bytesOf(pages: Int): ByteArray {
        val pageObjects = (0 until pages).map { FIRST_PAGE_OBJECT + 2 * it }
        val objects = mutableListOf(
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [${pageObjects.joinToString(separator = " ") { "$it 0 R" }}] /Count $pages >>",
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        )
        pageObjects.forEachIndexed { index, pageObject ->
            val content = "BT /F1 12 Tf 20 50 Td (Page ${index + 1}) Tj ET"
            objects += "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] " +
                "/Resources << /Font << /F1 3 0 R >> >> /Contents ${pageObject + 1} 0 R >>"
            objects += "<< /Length ${content.length} >>\nstream\n$content\nendstream"
        }
        return assemble(objects = objects).toByteArray(charset = Charsets.US_ASCII)
    }

    /** Numbers the objects from 1 and adds the cross-reference table with their byte offsets. */
    private fun assemble(objects: List<String>): String {
        val text = StringBuilder("%PDF-1.4\n")
        val offsets = objects.mapIndexed { index, body ->
            val offset = text.length
            text.append("${index + 1} 0 obj\n$body\nendobj\n")
            offset
        }
        val xref = text.length
        text.append("xref\n0 ${objects.size + 1}\n0000000000 65535 f \n")
        offsets.forEach { text.append("%010d 00000 n \n".format(it)) }
        text.append("trailer\n<< /Size ${objects.size + 1} /Root 1 0 R >>\nstartxref\n$xref\n%%EOF\n")
        return text.toString()
    }
}
