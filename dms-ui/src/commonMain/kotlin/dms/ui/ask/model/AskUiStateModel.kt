package dms.ui.ask.model

/**
 * The Ask screen: the conversation so far, the scopes to ask in (the whole library and the smart
 * folders), the chosen one, and what the AI lane runs now ([aiBusy]).
 */
data class AskUiStateModel(
    val entries: List<AskEntryModel> = emptyList(),
    val scopes: List<String> = listOf(WHOLE_LIBRARY),
    val scopeIndex: Int = 0,
    val aiBusy: String? = null,
) {
    companion object {
        const val WHOLE_LIBRARY: String = "Whole library"
    }
}
