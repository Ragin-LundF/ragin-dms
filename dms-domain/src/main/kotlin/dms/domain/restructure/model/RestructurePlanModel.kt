package dms.domain.restructure.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * What a restructure would do: the [moves], the documents already in place ([unchanged]) and those
 * left where they are ([skipped]). [renamedForClash] are the moves whose name got a ` (2)` suffix
 * because another document already has that name in the target folder.
 */
data class RestructurePlanModel(
    val template: String,
    val moves: List<PlannedMoveModel>,
    val unchanged: List<DocumentIdModel>,
    val skipped: List<SkippedDocumentModel>,
    val renamedForClash: Set<DocumentIdModel> = emptySet(),
) {
    /** The folders the documents leave, which may be empty afterwards. */
    val sourceFolders: Set<RelativePathModel>
        get() {
            return moves.mapNotNull { it.from.parent }.filterNot { it.isRoot }.toSet()
        }

    /** This plan with only the moves [keep] accepts, for example after the user unticked a subtree. */
    fun only(keep: (PlannedMoveModel) -> Boolean): RestructurePlanModel {
        val kept = moves.filter(predicate = keep)
        return copy(
            moves = kept,
            renamedForClash = renamedForClash.filter { id -> kept.any { it.documentId == id } }.toSet(),
        )
    }
}
