package dms.application.restructure

import dms.application.library.CatalogStore
import dms.application.port.plan.PlanStore
import dms.application.restructure.model.RestructureOutcome
import dms.domain.restructure.model.RestructurePlanModel

/**
 * Undoes an applied restructure: every document that is still where the restructure put it goes back
 * where it came from. Documents moved or renamed since are left alone and counted as skipped.
 */
class ReverseRestructure(
    private val store: CatalogStore,
    private val plans: PlanStore,
    private val apply: ApplyRestructurePlan,
) {

    suspend operator fun invoke(
        name: String,
        onProgress: (done: Int, total: Int) -> Unit = { _, _ -> },
    ): RestructureOutcome {
        val library = store.current() ?: return RestructureOutcome.LibraryNotOpen
        val record = plans.read(folder = library.folders.primary, name = name)
            ?: return RestructureOutcome.RecordNotFound
        val (unchanged, changed) = record.moves.partition { move ->
            library.catalog.documents[move.documentId]?.path == move.to
        }
        val inverse = RestructurePlanModel(
            template = "${record.template} (reversed)",
            moves = unchanged.map { it.inverse() }.reversed(),
            unchanged = emptyList(),
            skipped = emptyList(),
        )
        return apply(plan = inverse, skipped = changed.size, onProgress = onProgress)
    }

    /** The applied restructures that can be reversed, newest first. */
    suspend fun history(): List<String> {
        val library = store.current() ?: return emptyList()
        return plans.names(folder = library.folders.primary)
    }
}
