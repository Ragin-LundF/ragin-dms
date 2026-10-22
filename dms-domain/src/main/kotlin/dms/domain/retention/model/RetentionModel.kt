package dms.domain.retention.model

import java.time.Instant
import java.time.LocalDate

/**
 * How long the user wants to keep one document. Strictly opt-in: a document has no retention unless
 * the user sets [keepUntil] here or a period on its category. Once the user marked it done
 * ([settledAt]), the document is never flagged again, also not through its category.
 */
data class RetentionModel(
    val keepUntil: LocalDate? = null,
    val settledAt: Instant? = null,
) {
    val isEmpty: Boolean
        get() {
            return keepUntil == null && settledAt == null
        }
}
