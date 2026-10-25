package dms.storage.fs

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.TemporaryNames
import java.nio.ByteBuffer
import java.nio.channels.FileChannel
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.FileAlreadyExistsException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.StandardOpenOption
import java.util.UUID

/**
 * Writes that never leave a half-written file behind: everything goes to a `.dms-tmp-` file in the
 * target directory first and is then moved into place in one step.
 */
object AtomicFiles {

    /**
     * Copies [source] to [target], which must not exist, verifies the copy by hash and keeps the
     * modification time. An entry that appears at [target] meanwhile is never replaced: the result is
     * [StorageFailure.ALREADY_EXISTS].
     */
    fun copy(source: Path, target: Path): StorageResult<FileFactsModel> {
        Files.createDirectories(target.parent)
        val temp = tempFileFor(target = target)
        try {
            val sourceHash = ContentHasher.copyAndHash(source = source, target = temp)
            if (ContentHasher.hash(path = temp) != sourceHash) {
                return StorageResult.Failure(reason = StorageFailure.VERIFICATION_FAILED)
            }
            Files.setLastModifiedTime(temp, Files.getLastModifiedTime(source))
            // Hashing a large file takes long; check again for a clash that appeared meanwhile.
            if (PathResolver.clashing(directory = target.parent, name = target.fileName.toString()) != null) {
                return StorageResult.Failure(reason = StorageFailure.ALREADY_EXISTS)
            }
            val placed = moveWithoutReplacing(temp = temp, target = target)
            if (placed is StorageResult.Failure) {
                return placed
            }
            return StorageResult.Success(value = FileFactsModel(size = Files.size(target), contentHash = sourceHash))
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    /** Writes [bytes] to [target], replacing an existing file in one step. */
    fun write(target: Path, bytes: ByteArray) {
        Files.createDirectories(target.parent)
        val temp = tempFileFor(target = target)
        try {
            FileChannel.open(temp, StandardOpenOption.WRITE, StandardOpenOption.CREATE_NEW).use { channel ->
                val buffer = ByteBuffer.wrap(bytes)
                while (buffer.hasRemaining()) {
                    channel.write(buffer)
                }
                channel.force(true)
            }
            moveReplacing(temp = temp, target = target)
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    /** A sibling name for a temporary file or an intermediate rename. */
    fun tempFileFor(target: Path): Path {
        return target.resolveSibling(TemporaryNames.PREFIX + UUID.randomUUID())
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

    /** Replaces [target] with [temp] in one step, where the file system supports it. */
    private fun moveReplacing(temp: Path, target: Path) {
        try {
            Files.move(temp, target, StandardCopyOption.ATOMIC_MOVE)
        } catch (ignored: AtomicMoveNotSupportedException) {
            Files.move(temp, target, StandardCopyOption.REPLACE_EXISTING)
        }
    }
}
