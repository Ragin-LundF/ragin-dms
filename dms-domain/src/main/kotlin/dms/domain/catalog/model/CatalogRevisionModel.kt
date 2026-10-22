package dms.domain.catalog.model

/**
 * Counts the changes of a catalog. It decides which of two catalog copies is newer — never the file
 * modification time, which sync clients rewrite.
 */
@JvmInline
value class CatalogRevisionModel(val value: Long) : Comparable<CatalogRevisionModel> {
    init {
        require(value = value >= 0) { "Revision must not be negative" }
    }

    fun next(): CatalogRevisionModel {
        return CatalogRevisionModel(value = value + 1)
    }

    override fun compareTo(other: CatalogRevisionModel): Int {
        return value.compareTo(other = other.value)
    }

    override fun toString(): String {
        return value.toString()
    }

    companion object {
        val INITIAL: CatalogRevisionModel = CatalogRevisionModel(value = 0)
    }
}
