package dms.domain.restructure.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/** One document of a restructure, moving [from] its current path [to] its new one. */
data class PlannedMoveModel(
    val documentId: DocumentIdModel,
    val from: RelativePathModel,
    val to: RelativePathModel,
) {
    /** The move that undoes this one. */
    fun inverse(): PlannedMoveModel {
        return PlannedMoveModel(documentId = documentId, from = to, to = from)
    }
}
