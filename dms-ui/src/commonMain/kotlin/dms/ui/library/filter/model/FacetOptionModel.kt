package dms.ui.library.filter.model

/**
 * One value a facet chip offers: choosing it applies [filter]. [depth] indents subcategories and
 * subfolders; [count] is how many documents of the view have the value. [missing] marks "Without a
 * category", which the popover shows in the warning tint.
 */
data class FacetOptionModel(
    val filter: FacetFilter,
    val label: String,
    val count: Int,
    val depth: Int = 0,
    val missing: Boolean = false,
)
