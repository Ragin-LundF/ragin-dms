package dms.ui.dashboard.attention

import dms.domain.document.model.DocumentIdModel

/** The answers to the rows of "Needs your attention" that are not open tasks. */
interface AttentionHandler {

    fun openInbox()

    fun analyzeInbox()

    /** Opens the document in the library. */
    fun openDocument(documentId: DocumentIdModel)

    fun findScans()

    fun makeScansSearchable()

    fun exportCalendar()
}
