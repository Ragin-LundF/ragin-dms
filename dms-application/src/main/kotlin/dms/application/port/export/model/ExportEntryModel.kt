package dms.application.port.export.model

import dms.domain.library.model.RelativePathModel

/** One file of an export: the document at [source] in the library, written to [target] in the bundle. */
data class ExportEntryModel(
    val source: RelativePathModel,
    val target: String,
)
