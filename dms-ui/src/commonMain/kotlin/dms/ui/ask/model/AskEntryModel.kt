package dms.ui.ask.model

import dms.application.ask.model.CitationModel

/** One question and what came back: the answer with its sources, or a note why there is none. */
data class AskEntryModel(
    val question: String,
    val answer: String? = null,
    val citations: List<CitationModel> = emptyList(),
    val note: String? = null,
    val asking: Boolean = false,
)
