package dms.ui.session.model

/**
 * How far the running operation is: [done] of [total] steps, where [total] is `null` while it is not
 * known yet. [step] names the current stage, [detail] the item being worked on.
 */
data class OperationProgressModel(
    val step: String,
    val done: Int = 0,
    val total: Int? = null,
    val detail: String? = null,
) {
    /** From 0 to 1, or `null` while the total is unknown. */
    val fraction: Float?
        get() {
            val all = total ?: return null
            return if (all == 0) 1f else done.toFloat() / all
        }
}
