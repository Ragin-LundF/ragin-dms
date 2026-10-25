package dms.application.restructure

import dms.application.library.CatalogStore
import dms.application.port.plan.PlanStore
import dms.application.restructure.model.RestructureHistoryOutcome

/** The applied restructures that can be reversed, newest first. */
class ListRestructureHistory(
    private val store: CatalogStore,
    private val plans: PlanStore,
) {

    suspend operator fun invoke(): RestructureHistoryOutcome {
        val library = store.current() ?: return RestructureHistoryOutcome.LibraryNotOpen
        return RestructureHistoryOutcome.Listed(names = plans.names(folder = library.folders.primary))
    }
}
