package dms.application.library.model.outcome

import dms.application.library.model.DashboardModel

sealed interface DashboardOutcome {

    data class Built(val dashboard: DashboardModel) : DashboardOutcome

    data object LibraryNotOpen : DashboardOutcome
}
