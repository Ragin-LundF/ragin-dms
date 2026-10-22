package dms.application.resolution

import dms.application.port.state.TaskStore
import dms.domain.task.model.TaskKeyModel

/** Hides a task from the dashboard without changing anything. The same change found again stays hidden. */
class DismissTask(private val tasks: TaskStore) {

    suspend operator fun invoke(key: TaskKeyModel) {
        tasks.dismiss(key = key)
    }
}
