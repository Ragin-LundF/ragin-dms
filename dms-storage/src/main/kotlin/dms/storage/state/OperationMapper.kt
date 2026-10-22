package dms.storage.state

import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.PendingOperationModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.storage.state.dto.PendingOperationDto
import java.time.Instant

/** Maps journal entries to and from `journal.yaml`. */
object OperationMapper {

    private const val COPY = "copy"
    private const val CREATE_FOLDER = "createFolder"
    private const val MOVE = "move"
    private const val TRASH = "trash"
    private const val COPY_TEXT = "copyText"
    private const val SAVE_CATALOG = "saveCatalog"
    private const val COPY_PLAN = "copyPlan"
    private const val COPY_INDEX = "copyIndex"

    fun toDto(operation: PendingOperationModel): PendingOperationDto {
        val base = PendingOperationDto(
            id = operation.id,
            folderId = operation.folderId.value,
            recordedAt = operation.recordedAt.toString(),
            type = SAVE_CATALOG,
        )
        return when (val folderOperation = operation.operation) {
            is FolderOperation.CopyFromPrimary -> base.copy(
                type = COPY,
                path = folderOperation.path.value,
                replaceExisting = folderOperation.replaceExisting,
            )

            is FolderOperation.CreateFolder -> base.copy(type = CREATE_FOLDER, path = folderOperation.path.value)
            is FolderOperation.Move -> base.copy(
                type = MOVE,
                from = folderOperation.from.value,
                to = folderOperation.to.value,
            )

            is FolderOperation.MoveToTrash -> base.copy(type = TRASH, path = folderOperation.path.value)
            is FolderOperation.CopyText -> base.copy(type = COPY_TEXT, documentId = folderOperation.documentId.value)
            is FolderOperation.CopyPlan -> base.copy(type = COPY_PLAN, path = folderOperation.name)
            is FolderOperation.CopyIndex -> base.copy(type = COPY_INDEX, documentId = folderOperation.documentId.value)
            FolderOperation.SaveCatalog -> base
        }
    }

    /** The journal entry, or `null` for an entry this version does not understand. */
    fun toDomain(dto: PendingOperationDto): PendingOperationModel? {
        val operation = operationOf(dto = dto) ?: return null
        return PendingOperationModel(
            id = dto.id,
            folderId = LibraryFolderIdModel(value = dto.folderId),
            operation = operation,
            recordedAt = Instant.parse(dto.recordedAt),
        )
    }

    private fun operationOf(dto: PendingOperationDto): FolderOperation? {
        return when (dto.type) {
            COPY, CREATE_FOLDER, TRASH -> dto.path?.let {
                pathOperation(type = dto.type, path = RelativePathModel.of(raw = it), dto = dto)
            }

            MOVE -> move(from = dto.from, to = dto.to)
            COPY_TEXT, COPY_INDEX -> dto.documentId?.let {
                documentOperation(type = dto.type, documentId = DocumentIdModel(value = it))
            }

            COPY_PLAN -> dto.path?.let { FolderOperation.CopyPlan(name = it) }
            SAVE_CATALOG -> FolderOperation.SaveCatalog
            else -> null
        }
    }

    /** The operations on the derived data of one document. */
    private fun documentOperation(type: String, documentId: DocumentIdModel): FolderOperation {
        return if (type == COPY_TEXT) FolderOperation.CopyText(documentId = documentId) else FolderOperation.CopyIndex(
            documentId = documentId,
        )
    }

    /** The operations on one path. */
    private fun pathOperation(type: String, path: RelativePathModel, dto: PendingOperationDto): FolderOperation {
        return when (type) {
            COPY -> FolderOperation.CopyFromPrimary(path = path, replaceExisting = dto.replaceExisting == true)
            CREATE_FOLDER -> FolderOperation.CreateFolder(path = path)
            else -> FolderOperation.MoveToTrash(path = path)
        }
    }

    private fun move(from: String?, to: String?): FolderOperation? {
        if (from == null || to == null) {
            return null
        }
        return FolderOperation.Move(from = RelativePathModel.of(raw = from), to = RelativePathModel.of(raw = to))
    }
}
