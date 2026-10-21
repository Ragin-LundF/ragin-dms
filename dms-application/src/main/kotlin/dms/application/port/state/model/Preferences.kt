package dms.application.port.state.model

/**
 * How the tool behaves on this device. [addNewFilesWhileScanning] catalogs new files of the primary
 * folder during a scan instead of offering each one on the dashboard.
 */
data class Preferences(
    val addNewFilesWhileScanning: Boolean = true,
)
