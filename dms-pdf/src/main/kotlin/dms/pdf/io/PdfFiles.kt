package dms.pdf.io

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.TemporaryNames
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.pdfbox.Loader
import org.apache.pdfbox.io.RandomAccessReadBufferedFile
import org.apache.pdfbox.pdmodel.PDDocument
import java.io.IOException
import java.nio.file.FileAlreadyExistsException
import java.nio.file.Files
import java.nio.file.NoSuchFileException
import java.nio.file.Path
import java.security.MessageDigest
import java.util.HexFormat
import java.util.UUID

/**
 * Loading and saving PDFs safely: a new file is written to a `.dms-tmp-` file next to its target and
 * moved into place in one step, and never over an existing file. I/O runs on the IO dispatcher, and
 * expected failures become [StorageResult.Failure].
 */
object PdfFiles {

    private const val BUFFER_SIZE = 64 * 1024

    suspend fun <T> io(block: () -> StorageResult<T>): StorageResult<T> {
        return withContext(context = Dispatchers.IO) {
            try {
                block()
            } catch (exception: NoSuchFileException) {
                StorageResult.Failure(reason = StorageFailure.NOT_FOUND, detail = exception.message)
            } catch (exception: FileAlreadyExistsException) {
                StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS, detail = exception.message)
            } catch (exception: IOException) {
                // Also a PDF that PDFBox cannot parse or that is encrypted.
                StorageResult.Failure(reason = StorageFailure.UNSUPPORTED_FORMAT, detail = exception.message)
            }
        }
    }

    fun load(file: Path): PDDocument {
        return Loader.loadPDF(RandomAccessReadBufferedFile(file.toString()))
    }

    /** Saves [document] as [target], which must not exist yet, and returns the facts of the written file. */
    fun save(document: PDDocument, target: Path): StorageResult<FileFactsModel> {
        if (isTaken(target = target)) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        Files.createDirectories(target.parent)
        val temp = target.resolveSibling(TemporaryNames.PREFIX + UUID.randomUUID())
        try {
            document.save(temp.toFile())
            return place(temp = temp, target = target)
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    /**
     * Moves a finished [temp] file to [target], which must not exist yet, not even with another case.
     * The check runs right before the move, and the move itself never replaces anything.
     */
    fun place(temp: Path, target: Path): StorageResult<FileFactsModel> {
        if (isTaken(target = target)) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        Files.createDirectories(target.parent)
        val moved = moveWithoutReplacing(temp = temp, target = target)
        if (moved is StorageResult.Failure) {
            return moved
        }
        return StorageResult.Success(value = factsOf(file = target))
    }

    /**
     * Moves [temp] to [target] unless something is there. Never with `ATOMIC_MOVE`: on macOS and Linux
     * that is `rename(2)`, which replaces an existing target without asking. [temp] lies in the target's
     * directory, so the move is still a single rename.
     */
    internal fun moveWithoutReplacing(temp: Path, target: Path): StorageResult<Unit> {
        return try {
            Files.move(temp, target)
            StorageResult.Success(value = Unit)
        } catch (exception: FileAlreadyExistsException) {
            StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS, detail = exception.message)
        }
    }

    fun tempFor(target: Path): Path {
        return target.resolveSibling(TemporaryNames.PREFIX + UUID.randomUUID() + ".pdf")
    }

    /** Whether an entry next to [target] has its name, ignoring case and normalization. */
    private fun isTaken(target: Path): Boolean {
        val directory = target.parent
        if (!Files.isDirectory(directory)) {
            return false
        }
        val name = target.fileName.toString()
        return Files.newDirectoryStream(directory).use { entries ->
            entries.any { FileNamePolicy.isSameName(first = it.fileName.toString(), second = name) }
        }
    }

    fun factsOf(file: Path): FileFactsModel {
        val digest = MessageDigest.getInstance("SHA-256")
        Files.newInputStream(file).use { input ->
            val buffer = ByteArray(size = BUFFER_SIZE)
            var read = input.read(buffer)
            while (read >= 0) {
                digest.update(buffer, 0, read)
                read = input.read(buffer)
            }
        }
        return FileFactsModel(
            size = Files.size(file),
            contentHash = ContentHashModel(value = HexFormat.of().formatHex(digest.digest())),
        )
    }
}
