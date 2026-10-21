package dms.application.port.ai.model

import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.TagName
import java.time.LocalDate

/**
 * Everything the classifier gets for one document: its content and the library's existing taxonomy,
 * folders and file name examples, so it prefers what the library already uses.
 */
data class ClassificationRequest(
    val documentId: DocumentId,
    val currentFileName: String,
    val content: DocumentContent,
    val categories: List<Category>,
    val tags: Set<TagName>,
    val folders: List<RelativePath>,
    val fileNameExamples: List<String>,
    val today: LocalDate,
)
