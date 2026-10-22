package dms.application.classification.model

import dms.domain.taxonomy.TagName
import java.time.LocalDate

/**
 * The values of a suggestion the user approved, possibly edited. A `null` field was not approved and
 * leaves the document unchanged; [tags] are added to the document's tags.
 */
data class SuggestionApproval(
    val title: String? = null,
    val documentDate: LocalDate? = null,
    val correspondent: String? = null,
    val category: ApprovedCategory? = null,
    val tags: Set<TagName> = emptySet(),
    val fileName: String? = null,
)
