package dms.domain.catalog

/**
 * Counts the changes of a catalog. It decides which of two catalog copies is newer — never the file
 * modification time, which sync clients rewrite.
 */
@JvmInline
value class CatalogRevision(val value: Long) : Comparable<CatalogRevision> {
    init {
        require(value = value >= 0) { "Revision must not be negative" }
    }

    fun next(): CatalogRevision {
        return CatalogRevision(value = value + 1)
    }

    override fun compareTo(other: CatalogRevision): Int {
        return value.compareTo(other = other.value)
    }

    override fun toString(): String {
        return value.toString()
    }

    companion object {
        val INITIAL: CatalogRevision = CatalogRevision(value = 0)
    }
}
