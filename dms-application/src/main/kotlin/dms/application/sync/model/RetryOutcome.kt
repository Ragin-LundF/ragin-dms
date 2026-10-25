package dms.application.sync.model

sealed interface RetryOutcome {

    data class Retried(val report: RetryReportModel) : RetryOutcome

    data object LibraryNotOpen : RetryOutcome
}
