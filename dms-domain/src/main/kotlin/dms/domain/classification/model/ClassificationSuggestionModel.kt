package dms.domain.classification.model

import dms.domain.classification.SuggestedCategory
import dms.domain.document.model.DocumentIdModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.FileNamePolicy
import dms.domain.rule.match.RuleTarget
import java.time.LocalDate

/**
 * What the AI proposes for one document. A suggestion is a value, never applied state: nothing in
 * it reaches the catalog or the file name until the user approves it, field by field or as a whole.
 *
 * [targetFolder] is where the document could be filed, with a one-line [targetReason]; [fields] are
 * values for the library's custom fields; [ruleNotes] names the rule behind each part that came from
 * the matching rules instead of the AI.
 */
data class ClassificationSuggestionModel(
    val documentId: DocumentIdModel,
    val model: String,
    val title: String? = null,
    val category: SuggestedCategory? = null,
    val tags: List<SuggestedTagModel> = emptyList(),
    val documentDate: LocalDate? = null,
    val correspondent: String? = null,
    val fileName: String? = null,
    val transcription: String? = null,
    val confidence: Double? = null,
    val targetFolder: SuggestedFolderModel? = null,
    val targetReason: String? = null,
    val fields: Map<FieldKeyModel, FieldValue> = emptyMap(),
    val ruleNotes: Map<RuleTarget, String> = emptyMap(),
) {
    init {
        require(value = model.isNotBlank()) { "A suggestion must name its model" }
        require(value = fileName == null || FileNamePolicy.check(name = fileName) == null) {
            "Suggested file name violates the file name policy: $fileName"
        }
        require(value = confidence == null || confidence in 0.0..1.0) { "Confidence must be between 0 and 1" }
    }

    /** `true` if the matching rules alone made this suggestion, without the AI. */
    val byRules: Boolean
        get() {
            return model == RULES_MODEL
        }

    companion object {
        /** The model name of suggestions the matching rules made without the AI. */
        const val RULES_MODEL: String = "rules"
    }
}
