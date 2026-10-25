package dms.application.pdf.model

import dms.domain.document.model.DocumentIdModel

sealed interface ScansOutcome {

    /** The PDFs without a text layer, by path. */
    data class Found(val documentIds: List<DocumentIdModel>) : ScansOutcome

    data object LibraryNotOpen : ScansOutcome
}
