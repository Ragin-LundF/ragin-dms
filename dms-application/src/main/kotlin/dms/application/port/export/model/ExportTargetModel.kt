package dms.application.port.export.model

/**
 * Where an export goes on this device: a new folder [name] in [directory], or `name.zip` there with
 * [zip]. An existing folder or file of that name is never replaced; the writer picks a free name.
 */
data class ExportTargetModel(
    val directory: String,
    val name: String,
    val zip: Boolean,
) {
    init {
        require(value = directory.isNotBlank()) { "Export directory must not be blank" }
        require(value = name.isNotBlank()) { "Export name must not be blank" }
    }
}
