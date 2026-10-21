package dms.domain.classification

import dms.domain.document.DocumentId
import dms.domain.library.FileNamePolicy
import java.time.LocalDate

/**
 * What the AI proposes for one document. A suggestion is a value, never applied state: nothing in
 * it reaches the catalog or the file name until the user approves it, field by field or as a whole.
 */
data class ClassificationSuggestion(
    val documentId: DocumentId,
    val model: String,
    val title: String? = null,
    val category: SuggestedCategory? = null,
    val tags: List<SuggestedTag> = emptyList(),
    val documentDate: LocalDate? = null,
    val correspondent: String? = null,
    val fileName: String? = null,
    val transcription: String? = null,
    val confidence: Double? = null,
) {
    init {
        require(value = model.isNotBlank()) { "A suggestion must name its model" }
        require(value = fileName == null || FileNamePolicy.check(name = fileName) == null) {
            "Suggested file name violates the file name policy: $fileName"
        }
        require(value = confidence == null || confidence in 0.0..1.0) { "Confidence must be between 0 and 1" }
    }
}
