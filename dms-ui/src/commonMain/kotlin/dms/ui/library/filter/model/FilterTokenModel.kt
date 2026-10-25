package dms.ui.library.filter.model

/**
 * One condition in the search field, shown as `key: value`. A token of an active [filter] can be
 * removed; one of a saved view's own conditions has no [filter] and stays.
 */
data class FilterTokenModel(
    val key: String,
    val value: String,
    val filter: FacetFilter? = null,
) {
    val label: String
        get() {
            return "$key: $value"
        }
}
