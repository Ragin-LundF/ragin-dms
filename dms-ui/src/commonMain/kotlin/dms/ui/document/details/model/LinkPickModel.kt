package dms.ui.document.details.model

import dms.domain.document.model.DocumentIdModel

/** The open "Link to…" picker of one document: what the user searches for, and the matches. */
data class LinkPickModel(
    val documentId: DocumentIdModel,
    val query: String = "",
    val matches: List<LinkCandidateModel> = emptyList(),
    val label: String = "",
)
