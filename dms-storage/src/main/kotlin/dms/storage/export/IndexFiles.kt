package dms.storage.export

import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.IndexRowModel
import java.net.URLEncoder
import java.nio.charset.StandardCharsets

/**
 * The index of an export as `index.csv` and `index.html`. The CSV is UTF-8 with a byte order mark,
 * separated by `;` and quoted where needed, so spreadsheet applications read umlauts and line breaks
 * right. The HTML is one plain table with links to the files.
 */
object IndexFiles {

    private const val SEPARATOR = ";"
    private val BYTE_ORDER_MARK = byteArrayOf(0xEF.toByte(), 0xBB.toByte(), 0xBF.toByte())
    private val FIXED_COLUMNS = listOf("Path", "Title", "Date", "Correspondent", "Category", "Tags")

    fun csv(bundle: ExportBundleModel, rows: List<IndexRowModel>): ByteArray {
        val header = FIXED_COLUMNS + bundle.fieldLabels + "Note"
        val lines = listOf(header) + rows.map { valuesOf(row = it) }
        val text = lines.joinToString(separator = "\r\n", postfix = "\r\n") { line ->
            line.joinToString(separator = SEPARATOR) { csvField(value = it) }
        }
        return BYTE_ORDER_MARK + text.toByteArray(StandardCharsets.UTF_8)
    }

    fun html(bundle: ExportBundleModel, rows: List<IndexRowModel>): ByteArray {
        val header = (FIXED_COLUMNS + bundle.fieldLabels + "Note")
            .joinToString(separator = "") { "<th>${escaped(text = it)}</th>" }
        val body = rows.joinToString(separator = "\n") { row ->
            val values = valuesOf(row = row)
            val href = escaped(text = encodedPath(path = row.path))
            val link = "<td><a href=\"$href\">${escaped(text = row.path)}</a></td>"
            "<tr>$link" + values.drop(n = 1).joinToString(separator = "") { "<td>${escaped(text = it)}</td>" } + "</tr>"
        }
        val page = """
            <!DOCTYPE html>
            <html><head><meta charset="utf-8"><title>Documents</title>
            <style>body{font-family:sans-serif}table{border-collapse:collapse}
            td,th{border:1px solid #ccc;padding:4px 8px;vertical-align:top;white-space:pre-wrap}</style>
            </head><body><table>
            <tr>$header</tr>
            $body
            </table></body></html>
        """.trimIndent()
        return page.toByteArray(StandardCharsets.UTF_8)
    }

    private fun valuesOf(row: IndexRowModel): List<String> {
        return listOf(row.path, row.title, row.date, row.correspondent, row.category, row.tags) + row.fields + row.note
    }

    /** RFC 4180 quoting: a field with the separator, a quote or a line break is quoted, quotes doubled. */
    private fun csvField(value: String): String {
        val needsQuotes = value.contains(other = SEPARATOR) || value.contains(char = '"') ||
            value.contains(char = '\n') || value.contains(char = '\r') || value.contains(char = ',')
        return if (needsQuotes) "\"${value.replace(oldValue = "\"", newValue = "\"\"")}\"" else value
    }

    private fun escaped(text: String): String {
        return text.replace(oldValue = "&", newValue = "&amp;")
            .replace(oldValue = "<", newValue = "&lt;")
            .replace(oldValue = ">", newValue = "&gt;")
            .replace(oldValue = "\"", newValue = "&quot;")
    }

    private fun encodedPath(path: String): String {
        return path.split('/').joinToString(separator = "/") { segment ->
            URLEncoder.encode(segment, StandardCharsets.UTF_8).replace(oldValue = "+", newValue = "%20")
        }
    }
}
