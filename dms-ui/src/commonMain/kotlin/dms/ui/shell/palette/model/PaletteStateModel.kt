package dms.ui.shell.palette.model

/** What the palette shows: whether it is [open], the typed [query], its groups and the [selected] line. */
data class PaletteStateModel(
    val open: Boolean = false,
    val query: String = "",
    val groups: List<PaletteGroupModel> = emptyList(),
    val selected: Int = 0,
) {
    /** Every line of every group, in the order the arrow keys walk them. */
    val items: List<PaletteItemModel>
        get() {
            return groups.flatMap { it.items }
        }
}
