package dms.domain.query.model

import dms.domain.taxonomy.model.TagNameModel

/** Tag conditions: a document needs every tag of [all], at least one of [any], and none of [none]. */
data class TagFilterModel(
    val all: Set<TagNameModel> = emptySet(),
    val any: Set<TagNameModel> = emptySet(),
    val none: Set<TagNameModel> = emptySet(),
) {
    val isEmpty: Boolean
        get() {
            return all.isEmpty() && any.isEmpty() && none.isEmpty()
        }

    fun matches(tags: Set<TagNameModel>): Boolean {
        return tags.containsAll(elements = all) &&
            (any.isEmpty() || any.any { it in tags }) &&
            none.none { it in tags }
    }

    companion object {
        val NONE: TagFilterModel = TagFilterModel()
    }
}
