package dms.ui.document.details.model

import dms.domain.document.model.DocumentIdModel

/** A document to link to or a linked document, by its title or name, with the link's label. */
data class LinkCandidateModel(
    val documentId: DocumentIdModel,
    val name: String,
    val path: String,
    val label: String? = null,
)
