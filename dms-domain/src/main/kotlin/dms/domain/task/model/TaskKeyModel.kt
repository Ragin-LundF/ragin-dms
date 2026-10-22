package dms.domain.task.model

/**
 * Stable identity of an open task, derived from what the task is about. The same change found by
 * two scans yields the same key, so a dismissed task stays dismissed.
 */
@JvmInline
value class TaskKeyModel(val value: String) {
    override fun toString(): String {
        return value
    }
}
