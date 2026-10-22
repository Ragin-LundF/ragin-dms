package dms.ui.restructure.model

/**
 * The planned moves into one target [folder]: how many, a few of them as `old → new` samples, and
 * whether the user keeps them in the plan.
 */
data class MoveGroupModel(
    val folder: String,
    val count: Int,
    val samples: List<String>,
    val included: Boolean,
)
