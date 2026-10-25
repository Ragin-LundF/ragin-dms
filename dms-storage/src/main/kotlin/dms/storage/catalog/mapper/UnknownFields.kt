package dms.storage.catalog.mapper

import dms.storage.catalog.dto.ExtensibleDto

/** Takes over the fields only newer versions know from the entries of the file that is replaced. */
object UnknownFields {

    /** Adds to each of [entries] the unknown fields of the entry in [previous] with the same [keyOf]. */
    fun <T : ExtensibleDto> carry(entries: List<T>, previous: List<T>, keyOf: (T) -> Any): List<T> {
        val previousByKey = previous.associateBy(keyOf)
        entries.forEach { entry -> carry(entry = entry, previous = previousByKey[keyOf(entry)]) }
        return entries
    }

    /** Adds the unknown fields of [previous] to [entry]; returns [entry]. */
    fun <T : ExtensibleDto> carry(entry: T?, previous: T?): T? {
        if (entry != null && previous != null) {
            entry.unknownFields += previous.unknownFields
        }
        return entry
    }

    fun hasAny(entry: ExtensibleDto?): Boolean {
        return entry?.unknownFields.orEmpty().isNotEmpty()
    }
}
