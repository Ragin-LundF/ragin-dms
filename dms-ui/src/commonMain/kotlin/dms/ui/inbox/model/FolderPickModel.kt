package dms.ui.inbox.model

import dms.domain.document.model.DocumentIdModel

/**
 * The open "Move to…" picker for one document: the [folder] path typed or picked (a new path creates
 * the folder) and the [fileName] to keep or change.
 */
data class FolderPickModel(
    val documentId: DocumentIdModel,
    val documentName: String,
    val folder: String,
    val fileName: String,
)
