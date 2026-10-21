package dms.storage.fs

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.FileFacts
import java.nio.ByteBuffer
import java.nio.channels.FileChannel
import java.nio.file.AtomicMoveNotSupportedException
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

    const val TEMP_PREFIX: String = ".dms-tmp-"

    /**
     * Copies [source] to [target], which must not exist, verifies the copy by hash and keeps the
     * modification time.
     */
    fun copy(source: Path, target: Path): StorageResult<FileFacts> {
        Files.createDirectories(target.parent)
        val temp = tempFileFor(target = target)
        try {
            val sourceHash = ContentHasher.copyAndHash(source = source, target = temp)
            if (ContentHasher.hash(path = temp) != sourceHash) {
                return StorageResult.Failure(reason = StorageFailure.VERIFICATION_FAILED)
            }
            Files.setLastModifiedTime(temp, Files.getLastModifiedTime(source))
            moveIntoPlace(temp = temp, target = target, replace = false)
            return StorageResult.Success(value = FileFacts(size = Files.size(target), contentHash = sourceHash))
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
            moveIntoPlace(temp = temp, target = target, replace = true)
        } finally {
            Files.deleteIfExists(temp)
        }
    }

    /** A sibling name for a temporary file or an intermediate rename. */
    fun tempFileFor(target: Path): Path {
        return target.resolveSibling(TEMP_PREFIX + UUID.randomUUID())
    }

    private fun moveIntoPlace(temp: Path, target: Path, replace: Boolean) {
        try {
            Files.move(temp, target, StandardCopyOption.ATOMIC_MOVE)
        } catch (ignored: AtomicMoveNotSupportedException) {
            if (replace) {
                Files.move(temp, target, StandardCopyOption.REPLACE_EXISTING)
            } else {
                Files.move(temp, target)
            }
        }
    }
}
