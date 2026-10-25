package dms.ui.library.selection.model

import dms.domain.document.model.DocumentIdModel

/** The documents ticked for a bulk action; [anchor] is where a Shift-click range starts. */
data class DocumentSelectionModel(
    val selected: Set<DocumentIdModel> = emptySet(),
    val anchor: DocumentIdModel? = null,
)
