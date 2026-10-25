package dms.ui.library.sidebar.model

/** The counts beside the sidebar's views; [inbox] and [needsReview] are badges in the warning tint. */
data class SidebarCountsModel(
    val all: Int = 0,
    val inbox: Int = 0,
    val needsReview: Int = 0,
    val addedThisMonth: Int = 0,
    val folders: Int = 0,
)
