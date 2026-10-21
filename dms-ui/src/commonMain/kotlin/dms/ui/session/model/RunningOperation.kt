package dms.ui.session.model

/** An operation that runs now, in its [lane]; [cancellable] tells whether the user may stop it. */
data class RunningOperation(
    val label: String,
    val lane: OperationLane,
    val cancellable: Boolean,
)
