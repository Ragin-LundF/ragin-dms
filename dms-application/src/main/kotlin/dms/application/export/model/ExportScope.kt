package dms.application.export.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.query.model.DocumentQueryModel

/** Which documents an export takes. */
sealed interface ExportScope {

    data class Documents(val ids: List<DocumentIdModel>) : ExportScope

    /** The documents of a smart folder, a preset such as a tax year, or any other query. */
    data class Query(val query: DocumentQueryModel) : ExportScope
}
