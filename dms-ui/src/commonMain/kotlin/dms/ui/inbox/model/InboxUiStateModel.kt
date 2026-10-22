package dms.ui.inbox.model

/**
 * The inbox screen. [folders] are the library's folders to file into, [recentTargets] the folders
 * the user filed into last. [busy] and [aiBusy] name what runs in the library and the AI lane.
 */
data class InboxUiStateModel(
    val enabled: Boolean = false,
    val path: String = "",
    val aiAnalysis: Boolean = true,
    val rows: List<InboxRowModel> = emptyList(),
    val folders: List<String> = emptyList(),
    val recentTargets: List<String> = emptyList(),
    val pick: FolderPickModel? = null,
    val busy: String? = null,
    val aiBusy: String? = null,
)
