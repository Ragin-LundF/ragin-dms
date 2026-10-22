package dms.storage.text

import dms.application.port.storage.TextStore
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.storage.fs.AtomicFiles
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path

/**
 * The text of documents in `.dms-text/<documentId>.txt`, UTF-8. Named by id, so renaming or moving a
 * document never touches it. Derived data: a missing file is not an error.
 */
class FileTextStore : TextStore {

    override suspend fun write(
        folder: LibraryFolderModel,
        documentId: DocumentIdModel,
        text: String,
    ): StorageResult<Unit> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val file = fileOf(root = root, documentId = documentId)
            AtomicFiles.write(target = file, bytes = text.toByteArray(StandardCharsets.UTF_8))
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): String? {
        return withContext(context = Dispatchers.IO) {
            val root = PathResolver.root(folder = folder)
            val file = root?.let { fileOf(root = it, documentId = documentId) }
            try {
                file?.takeIf { Files.isRegularFile(it) }?.let { Files.readString(it, StandardCharsets.UTF_8) }
            } catch (ignored: IOException) {
                null
            }
        }
    }

    private fun fileOf(root: Path, documentId: DocumentIdModel): Path {
        return root.resolve(TEXT_FOLDER).resolve("${documentId.value}.txt")
    }

    companion object {
        const val TEXT_FOLDER: String = ".dms-text"
    }
}
