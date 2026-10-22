package dms.storage.index

import dms.application.port.index.VectorIndex
import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.index.model.IndexedChunkModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.storage.fs.AtomicFiles
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path

/**
 * `.dms-index/<documentId>.vec`: a small header (format, model, dimensions, content and metadata
 * hash, chunk count) and per chunk its offsets and float32 values. Derived data: a file that cannot be
 * read counts as missing and is simply rebuilt.
 */
class FileVectorIndex : VectorIndex {

    override suspend fun write(folder: LibraryFolderModel, vectors: DocumentVectorsModel): StorageResult<Unit> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            AtomicFiles.write(
                target = fileOf(root = root, documentId = vectors.documentId),
                bytes = encode(vectors = vectors),
            )
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): DocumentVectorsModel? {
        return withContext(context = Dispatchers.IO) {
            val file = PathResolver.root(folder = folder)?.let { fileOf(root = it, documentId = documentId) }
            try {
                file?.takeIf { Files.isRegularFile(it) }
                    ?.let { decode(documentId = documentId, bytes = Files.readAllBytes(it)) }
            } catch (ignored: IOException) {
                null
            } catch (ignored: IllegalArgumentException) {
                null
            }
        }
    }

    private fun encode(vectors: DocumentVectorsModel): ByteArray {
        val bytes = ByteArrayOutputStream()
        DataOutputStream(bytes).use { output ->
            output.writeInt(MAGIC)
            output.writeInt(FORMAT)
            output.writeUTF(vectors.model)
            output.writeInt(vectors.dimensions)
            output.writeUTF(vectors.contentHash.value)
            output.writeUTF(vectors.metadataHash)
            output.writeInt(vectors.chunks.size)
            vectors.chunks.forEach { chunk ->
                output.writeInt(chunk.start)
                output.writeInt(chunk.end)
                chunk.vector.forEach { output.writeFloat(it) }
            }
        }
        return bytes.toByteArray()
    }

    private fun decode(documentId: DocumentIdModel, bytes: ByteArray): DocumentVectorsModel? {
        DataInputStream(bytes.inputStream()).use { input ->
            if (input.readInt() != MAGIC || input.readInt() != FORMAT) {
                return null
            }
            val model = input.readUTF()
            val dimensions = input.readInt()
            val contentHash = ContentHashModel(value = input.readUTF())
            val metadataHash = input.readUTF()
            val chunks = List(size = input.readInt()) {
                val start = input.readInt()
                val end = input.readInt()
                IndexedChunkModel(
                    start = start,
                    end = end,
                    vector = FloatArray(size = dimensions) { input.readFloat() },
                )
            }
            return DocumentVectorsModel(
                documentId = documentId,
                model = model,
                dimensions = dimensions,
                contentHash = contentHash,
                metadataHash = metadataHash,
                chunks = chunks,
            )
        }
    }

    private fun fileOf(root: Path, documentId: DocumentIdModel): Path {
        return root.resolve(INDEX_FOLDER).resolve("${documentId.value}.vec")
    }

    companion object {
        const val INDEX_FOLDER: String = ".dms-index"
        private const val MAGIC = 0x444D5356
        private const val FORMAT = 1
    }
}
