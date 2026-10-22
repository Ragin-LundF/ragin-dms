package dms.ui.restructure.model

import dms.application.restructure.model.RestructureScope

/** A scope to restructure, as the picker names it: a folder or a smart folder. */
data class ScopeOptionModel(
    val label: String,
    val scope: RestructureScope,
)
