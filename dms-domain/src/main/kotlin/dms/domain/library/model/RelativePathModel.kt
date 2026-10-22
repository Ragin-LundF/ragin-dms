package dms.domain.library.model

import java.text.Normalizer

/**
 * A path inside a library folder, independent of where that folder lives on a device.
 *
 * Always `/`-separated, NFC-normalized and relative; the empty path is the library folder itself.
 * macOS may report names decomposed (NFD) while Windows, Linux and network shares use NFC, so
 * normalizing here is what makes the same file match across folders on different file systems.
 *
 * Segments are not checked against [FileNamePolicy]: a path describes what exists on disk, and a
 * file created elsewhere may carry a name the policy would not allow for a new file.
 */
@JvmInline
value class RelativePathModel private constructor(val value: String) {

    init {
        require(value = !value.startsWith(prefix = SEPARATOR)) { "Path must be relative: $value" }
        require(value = !DRIVE_PREFIX.containsMatchIn(input = value)) { "Path must be relative: $value" }
        if (value.isNotEmpty()) {
            require(value = value.split(SEPARATOR).none { it in INVALID_SEGMENTS }) {
                "Path must not contain empty, '.' or '..' segments: $value"
            }
        }
    }

    val isRoot: Boolean
        get() {
            return value.isEmpty()
        }

    val segments: List<String>
        get() {
            if (isRoot) {
                return emptyList()
            }
            return value.split(SEPARATOR)
        }

    /** The last segment, or an empty string for the root. */
    val fileName: String
        get() {
            return value.substringAfterLast(delimiter = SEPARATOR)
        }

    /** The containing folder, or `null` for the root. */
    val parent: RelativePathModel?
        get() {
            if (isRoot) {
                return null
            }
            return RelativePathModel(
                value = value.substringBeforeLast(delimiter = SEPARATOR, missingDelimiterValue = ""),
            )
        }

    fun resolve(name: String): RelativePathModel {
        if (isRoot) {
            return of(raw = name)
        }
        return of(raw = value + SEPARATOR + name)
    }

    /** `true` if this path lies strictly below [folder]. */
    fun isInside(folder: RelativePathModel): Boolean {
        if (this == folder) {
            return false
        }
        return folder.isRoot || value.startsWith(prefix = folder.value + SEPARATOR)
    }

    /** Moves this path from below [from] to below [to], keeping the part after [from]. */
    fun relocate(from: RelativePathModel, to: RelativePathModel): RelativePathModel {
        require(value = this == from || isInside(folder = from)) { "$value is not inside ${from.value}" }
        if (this == from) {
            return to
        }
        val remainder = if (from.isRoot) value else value.removePrefix(prefix = from.value + SEPARATOR)
        return to.resolve(name = remainder)
    }

    override fun toString(): String {
        return value
    }

    companion object {
        private const val SEPARATOR = "/"
        private val INVALID_SEGMENTS = setOf("", ".", "..")
        private val DRIVE_PREFIX = Regex(pattern = "^[A-Za-z]:")

        val ROOT: RelativePathModel = RelativePathModel(value = "")

        /** Parses a `/`-separated path as stored in the catalog. A trailing separator is ignored. */
        fun of(raw: String): RelativePathModel {
            return RelativePathModel(
                value = Normalizer.normalize(raw.removeSuffix(suffix = SEPARATOR), Normalizer.Form.NFC),
            )
        }

        /** Builds a path from file system segments, which may contain characters such as `\`. */
        fun fromSegments(segments: List<String>): RelativePathModel {
            return of(raw = segments.joinToString(separator = SEPARATOR))
        }
    }
}
