package dms.storage.state

import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.PendingOperation
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
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

    fun toDto(operation: PendingOperation): PendingOperationDto {
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
            FolderOperation.SaveCatalog -> base
        }
    }

    /** The journal entry, or `null` for an entry this version does not understand. */
    fun toDomain(dto: PendingOperationDto): PendingOperation? {
        val operation = when (dto.type) {
            COPY -> dto.path?.let {
                FolderOperation.CopyFromPrimary(
                    path = RelativePath.of(raw = it),
                    replaceExisting = dto.replaceExisting == true,
                )
            }

            CREATE_FOLDER -> dto.path?.let { FolderOperation.CreateFolder(path = RelativePath.of(raw = it)) }
            MOVE -> move(from = dto.from, to = dto.to)
            TRASH -> dto.path?.let { FolderOperation.MoveToTrash(path = RelativePath.of(raw = it)) }
            COPY_TEXT -> dto.documentId?.let { FolderOperation.CopyText(documentId = DocumentId(value = it)) }
            SAVE_CATALOG -> FolderOperation.SaveCatalog
            else -> null
        } ?: return null
        return PendingOperation(
            id = dto.id,
            folderId = LibraryFolderId(value = dto.folderId),
            operation = operation,
            recordedAt = Instant.parse(dto.recordedAt),
        )
    }

    private fun move(from: String?, to: String?): FolderOperation? {
        if (from == null || to == null) {
            return null
        }
        return FolderOperation.Move(from = RelativePath.of(raw = from), to = RelativePath.of(raw = to))
    }
}
