package dms.ui.inbox.model

import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel

/**
 * One document in the inbox: its name, where in the inbox it lies, when it arrived, and the AI's
 * suggestion if one waits. [analyzable] is `false` for files the AI never reads.
 */
data class InboxRowModel(
    val documentId: DocumentIdModel,
    val name: String,
    val subfolder: String,
    val added: String,
    val size: String,
    val suggestion: ClassificationSuggestionModel?,
    val analyzable: Boolean,
    val prefill: String?,
)
