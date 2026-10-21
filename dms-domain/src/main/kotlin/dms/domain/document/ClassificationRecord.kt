package dms.domain.document

import java.time.Instant

/** Provenance of a document's categorization: who suggested it, with which model, confirmed when. */
data class ClassificationRecord(
    val source: ClassificationSource,
    val model: String?,
    val confirmedAt: Instant,
) {
    init {
        require(value = source != ClassificationSource.AI || !model.isNullOrBlank()) {
            "An AI classification must name its model"
        }
    }
}
