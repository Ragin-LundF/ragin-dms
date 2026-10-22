package dms.domain.document.model

import java.time.LocalDate

/** Descriptive data about a document, edited by the user or filled from an approved suggestion. */
data class DocumentMetadataModel(
    val title: String? = null,
    val documentDate: LocalDate? = null,
    val correspondent: String? = null,
) {
    init {
        require(value = title == null || title.isNotBlank()) { "Title must be null or not blank" }
        require(value = correspondent == null || correspondent.isNotBlank()) {
            "Correspondent must be null or not blank"
        }
    }

    companion object {
        val EMPTY: DocumentMetadataModel = DocumentMetadataModel()
    }
}
