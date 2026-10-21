package dms.ai.support

import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.DocumentContent
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import java.time.LocalDate

/** Classification requests with synthetic names only. */
object Requests {

    val invoices: Category = Category(id = CategoryId(value = "invoices"), name = "Invoices")
    val energy: Category = Category(id = CategoryId(value = "energy"), name = "Energy", parentId = invoices.id)

    fun request(
        content: DocumentContent = DocumentContent.Text(text = "Sample text"),
        currentFileName: String = "scan-001.pdf",
    ): ClassificationRequest {
        return ClassificationRequest(
            documentId = DocumentId(value = "doc-1"),
            currentFileName = currentFileName,
            content = content,
            categories = listOf(invoices, energy),
            tags = setOf(TagName.of(raw = "Tax")),
            folders = listOf(RelativePath.of(raw = "letters")),
            fileNameExamples = listOf(
                "2024-01-02 Sample - One.pdf",
                "2024-02-03 Sample - Two.pdf",
                "2024-03-04 Sample - Three.pdf",
            ),
            today = LocalDate.parse("2026-01-15"),
        )
    }
}
