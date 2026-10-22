package dms.ui.assistant.model

import dms.domain.document.model.DocumentModel

/** The summary dialog: [working] while the AI reads, then the [text] from [model], or the [problem]. */
data class SummaryStateModel(
    val document: DocumentModel,
    val working: Boolean = true,
    val text: String? = null,
    val model: String? = null,
    val problem: String? = null,
)
