package dms.application.sync.model

/**
 * The result of retrying the journal. [dropped] operations had become pointless, for example a copy
 * of a file that no longer exists in the primary folder; the next scan shows what is left.
 */
data class RetryReport(
    val succeeded: Int = 0,
    val dropped: Int = 0,
    val stillPending: Int = 0,
) {

    operator fun plus(other: RetryReport): RetryReport {
        return RetryReport(
            succeeded = succeeded + other.succeeded,
            dropped = dropped + other.dropped,
            stillPending = stillPending + other.stillPending,
        )
    }
}
