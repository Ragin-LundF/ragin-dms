package dms.storage.fs

import dms.domain.document.ContentHash
import java.nio.channels.Channels
import java.nio.channels.FileChannel
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardOpenOption
import java.security.MessageDigest
import java.util.HexFormat

/** SHA-256 of file contents, computed while streaming so large files never sit in memory. */
object ContentHasher {

    private const val BUFFER_SIZE = 64 * 1024

    fun hash(path: Path): ContentHash {
        val digest = MessageDigest.getInstance("SHA-256")
        Files.newInputStream(path).use { input ->
            val buffer = ByteArray(size = BUFFER_SIZE)
            var read = input.read(buffer)
            while (read >= 0) {
                digest.update(buffer, 0, read)
                read = input.read(buffer)
            }
        }
        return toContentHash(digest = digest)
    }

    /**
     * Copies [source] into the new file [target], forces it to disk and returns the hash of what was
     * read. The caller verifies the written file against it.
     */
    fun copyAndHash(source: Path, target: Path): ContentHash {
        val digest = MessageDigest.getInstance("SHA-256")
        FileChannel.open(target, StandardOpenOption.WRITE, StandardOpenOption.CREATE_NEW).use { channel ->
            val output = Channels.newOutputStream(channel)
            Files.newInputStream(source).use { input ->
                val buffer = ByteArray(size = BUFFER_SIZE)
                var read = input.read(buffer)
                while (read >= 0) {
                    digest.update(buffer, 0, read)
                    output.write(buffer, 0, read)
                    read = input.read(buffer)
                }
            }
            output.flush()
            channel.force(true)
        }
        return toContentHash(digest = digest)
    }

    private fun toContentHash(digest: MessageDigest): ContentHash {
        return ContentHash(value = HexFormat.of().formatHex(digest.digest()))
    }
}
