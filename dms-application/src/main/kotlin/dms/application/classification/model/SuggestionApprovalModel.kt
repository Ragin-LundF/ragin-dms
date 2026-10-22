package dms.application.classification.model

import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.LocalDate

/**
 * The values of a suggestion the user approved, possibly edited. A `null` field was not approved and
 * leaves the document unchanged; [tags] are added to the document's tags. [targetFolder] files the
 * document there, together with the approved [fileName]. [fields] are set on the document, next to
 * the values it already has.
 */
data class SuggestionApprovalModel(
    val title: String? = null,
    val documentDate: LocalDate? = null,
    val correspondent: String? = null,
    val category: ApprovedCategory? = null,
    val tags: Set<TagNameModel> = emptySet(),
    val fileName: String? = null,
    val targetFolder: RelativePathModel? = null,
    val fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
)
