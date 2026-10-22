package dms.pdf.io

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.document.model.ContentHashModel
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
import java.nio.file.StandardCopyOption
import java.security.MessageDigest
import java.util.HexFormat
import java.util.UUID

/**
 * Loading and saving PDFs safely: a new file is written to a `.dms-tmp-` file next to its target and
 * moved into place in one step, and never over an existing file. I/O runs on the IO dispatcher, and
 * expected failures become [StorageResult.Failure].
 */
object PdfFiles {

    private const val TEMP_PREFIX = ".dms-tmp-"
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
        if (Files.exists(target)) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        Files.createDirectories(target.parent)
        val temp = target.resolveSibling(TEMP_PREFIX + UUID.randomUUID())
        try {
            document.save(temp.toFile())
            Files.move(temp, target, StandardCopyOption.ATOMIC_MOVE)
        } finally {
            Files.deleteIfExists(temp)
        }
        return StorageResult.Success(value = factsOf(file = target))
    }

    /** Moves a finished [temp] file to [target], which must not exist yet. */
    fun place(temp: Path, target: Path): StorageResult<FileFactsModel> {
        if (Files.exists(target)) {
            return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        Files.createDirectories(target.parent)
        Files.move(temp, target, StandardCopyOption.ATOMIC_MOVE)
        return StorageResult.Success(value = factsOf(file = target))
    }

    fun tempFor(target: Path): Path {
        return target.resolveSibling(TEMP_PREFIX + UUID.randomUUID() + ".pdf")
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
