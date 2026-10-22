package dms.application.port.plan.model

import dms.domain.restructure.model.PlannedMoveModel
import java.time.Instant

/**
 * A restructure as it was applied: the moves that were done, kept in `.dms-plans/` so the user can
 * reverse it later. [name] identifies the file, [template] says what the user chose.
 */
data class RestructureRecordModel(
    val name: String,
    val appliedAt: Instant,
    val template: String,
    val moves: List<PlannedMoveModel>,
)
