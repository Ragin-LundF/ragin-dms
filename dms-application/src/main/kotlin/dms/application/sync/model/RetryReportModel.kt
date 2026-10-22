package dms.application.sync.model

/**
 * The result of retrying the journal. [dropped] operations had become pointless, for example a copy
 * of a file that no longer exists in the primary folder; the next scan shows what is left.
 */
data class RetryReportModel(
    val succeeded: Int = 0,
    val dropped: Int = 0,
    val stillPending: Int = 0,
) {

    operator fun plus(other: RetryReportModel): RetryReportModel {
        return RetryReportModel(
            succeeded = succeeded + other.succeeded,
            dropped = dropped + other.dropped,
            stillPending = stillPending + other.stillPending,
        )
    }
}
