package dms.ui.export.model

import dms.application.export.model.ExportOptionsModel
import dms.application.export.model.ExportScope

/** The export the user is setting up: which documents ([scopeLabel] names them), its name and layout. */
data class ExportDraftModel(
    val scope: ExportScope,
    val scopeLabel: String,
    val name: String,
    val options: ExportOptionsModel = ExportOptionsModel(),
    val correspondent: String = "",
)
