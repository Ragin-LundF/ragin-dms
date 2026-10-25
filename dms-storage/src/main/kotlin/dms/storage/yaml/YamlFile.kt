package dms.storage.yaml

import dms.storage.fs.AtomicFiles
import java.nio.file.Files
import java.nio.file.NoSuchFileException
import java.nio.file.Path

/** One YAML file holding a value of [type], written atomically. Callers handle I/O and format errors. */
class YamlFile<T : Any>(
    private val path: Path,
    private val type: Class<T>,
) {

    /** The stored value, or `null` if the file does not exist (yet, or any more). */
    fun read(): T? {
        // Read first instead of asking whether it exists: the file can go between the question and the read.
        val bytes = try {
            Files.readAllBytes(path)
        } catch (ignored: NoSuchFileException) {
            return null
        }
        return YamlMappers.yaml.readValue(bytes, type)
    }

    fun write(value: T) {
        AtomicFiles.write(target = path, bytes = YamlMappers.yaml.writeValueAsBytes(value))
    }
}
