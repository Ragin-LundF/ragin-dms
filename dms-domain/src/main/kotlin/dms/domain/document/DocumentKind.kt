package dms.domain.document

import dms.domain.library.RelativePath
import java.util.Locale

/**
 * What kind of content a file holds, decided by its extension.
 *
 * [OTHER] files are catalogued and synchronized like every other file, but never sent to the AI.
 */
enum class DocumentKind(val readableByAi: Boolean) {
    PDF(readableByAi = true),
    IMAGE(readableByAi = true),
    OFFICE(readableByAi = true),
    OTHER(readableByAi = false),
    ;

    companion object {
        private val BY_EXTENSION = mapOf(
            "pdf" to PDF,
            "jpg" to IMAGE,
            "jpeg" to IMAGE,
            "png" to IMAGE,
            "docx" to OFFICE,
            "pptx" to OFFICE,
            "xlsx" to OFFICE,
        )

        fun of(path: RelativePath): DocumentKind {
            val extension = path.fileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
            return BY_EXTENSION[extension.lowercase(Locale.ROOT)] ?: OTHER
        }
    }
}
