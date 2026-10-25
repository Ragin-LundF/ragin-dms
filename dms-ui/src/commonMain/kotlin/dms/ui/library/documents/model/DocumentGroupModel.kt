package dms.ui.library.documents.model

import dms.ui.library.model.DocumentRowModel

/** The documents under one group header, such as "October 2025". */
data class DocumentGroupModel(
    val title: String,
    val rows: List<DocumentRowModel>,
)
