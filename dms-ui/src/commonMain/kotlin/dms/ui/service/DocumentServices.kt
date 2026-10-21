package dms.ui.service

import dms.application.catalog.EditCatalog
import dms.application.document.AddDocuments
import dms.application.document.AdoptNewFile
import dms.application.document.MoveDocument
import dms.application.document.RestoreDocument
import dms.application.document.TrashDocument
import dms.application.document.TrashDuplicateFile
import dms.application.document.TrashNewFile
import dms.application.folder.CreateFolder
import dms.application.folder.MoveFolder

/** Use cases that change documents, folders and their categorization. */
class DocumentServices(
    val addDocuments: AddDocuments,
    val adoptNewFile: AdoptNewFile,
    val moveDocument: MoveDocument,
    val trashDocument: TrashDocument,
    val trashNewFile: TrashNewFile,
    val trashDuplicateFile: TrashDuplicateFile,
    val restoreDocument: RestoreDocument,
    val createFolder: CreateFolder,
    val moveFolder: MoveFolder,
    val editCatalog: EditCatalog,
)
