package dms.ui.dashboard.model

/** One library folder on the dashboard. */
data class FolderStatusModel(
    val name: String,
    val location: String,
    val isPrimary: Boolean,
    val health: FolderHealth,
    val pendingOperations: Int,
)
