package dms.storage.yaml

import dms.storage.fs.AtomicFiles
import java.nio.file.Files
import java.nio.file.Path

/** One YAML file holding a value of [type], written atomically. Callers handle I/O and format errors. */
class YamlFile<T : Any>(
    private val path: Path,
    private val type: Class<T>,
) {

    /** The stored value, or `null` if the file does not exist yet. */
    fun read(): T? {
        if (!Files.exists(path)) {
            return null
        }
        return YamlMappers.yaml.readValue(Files.readAllBytes(path), type)
    }

    fun write(value: T) {
        AtomicFiles.write(target = path, bytes = YamlMappers.yaml.writeValueAsBytes(value))
    }
}
