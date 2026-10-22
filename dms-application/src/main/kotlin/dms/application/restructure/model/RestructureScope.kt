package dms.application.restructure.model

import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel

/** Which documents a restructure looks at. */
sealed interface RestructureScope {

    /** Every document directly or further below [path]; the root means the whole library. */
    data class Folder(val path: RelativePathModel) : RestructureScope

    /** The documents of a saved search or any other query. */
    data class Query(val query: DocumentQueryModel) : RestructureScope
}
