package dms.ui.assistant.model

import dms.domain.document.Document

/** The summary dialog: [working] while the AI reads, then the [text] from [model], or the [problem]. */
data class SummaryState(
    val document: Document,
    val working: Boolean = true,
    val text: String? = null,
    val model: String? = null,
    val problem: String? = null,
)
