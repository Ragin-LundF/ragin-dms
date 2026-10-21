package dms.domain.library

import java.text.Normalizer
import java.util.Locale

/**
 * The only validator for names of new files and folders, whether typed by the user or suggested by
 * the AI.
 *
 * A library is spread over macOS, Windows, Linux and network shares, so a name must be valid on all
 * of them: the strictest rules (Windows) apply everywhere. Names are also compared the way the
 * case-insensitive file systems compare them, so two siblings never differ only by case.
 */
object FileNamePolicy {

    /** Conservative limit that leaves room for the folder path within Windows' path length. */
    const val MAX_LENGTH: Int = 200

    private const val FIRST_PRINTABLE = 32
    private const val DELETE = 127
    private val FORBIDDEN_CHARACTERS = setOf('<', '>', ':', '"', '/', '\\', '|', '?', '*')
    private val RESERVED_NAMES = setOf("CON", "PRN", "AUX", "NUL") +
        (1..9).map { "COM$it" } +
        (1..9).map { "LPT$it" }

    /** Returns the first rule [name] breaks, or `null` if it is a valid name. */
    fun check(name: String): FileNameViolation? {
        return when {
            name.isEmpty() -> FileNameViolation.EMPTY
            name.length > MAX_LENGTH -> FileNameViolation.TOO_LONG
            name.any { isControlCharacter(character = it) } -> FileNameViolation.CONTROL_CHARACTER
            name.any { it in FORBIDDEN_CHARACTERS } -> FileNameViolation.FORBIDDEN_CHARACTER
            name.first() == '.' || name.first().isWhitespace() -> FileNameViolation.LEADING_DOT_OR_SPACE
            name.last() == '.' || name.last().isWhitespace() -> FileNameViolation.TRAILING_DOT_OR_SPACE
            isReserved(name = name) -> FileNameViolation.RESERVED_NAME
            else -> null
        }
    }

    /** `true` if a case-insensitive, Unicode-normalizing file system would treat both as one name. */
    fun isSameName(first: String, second: String): Boolean {
        return comparisonKey(name = first) == comparisonKey(name = second)
    }

    /** `true` if [name] would clash with one of [siblings] in the same folder. */
    fun collidesWithAny(name: String, siblings: Collection<String>): Boolean {
        return siblings.any { isSameName(first = it, second = name) }
    }

    private fun isControlCharacter(character: Char): Boolean {
        return character.code < FIRST_PRINTABLE || character.code == DELETE
    }

    private fun isReserved(name: String): Boolean {
        val baseName = name.substringBefore(delimiter = '.').trimEnd()
        return baseName.uppercase(Locale.ROOT) in RESERVED_NAMES
    }

    private fun comparisonKey(name: String): String {
        return Normalizer.normalize(name, Normalizer.Form.NFC).lowercase(Locale.ROOT)
    }
}
