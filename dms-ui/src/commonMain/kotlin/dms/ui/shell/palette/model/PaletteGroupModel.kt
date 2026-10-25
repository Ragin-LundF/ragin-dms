package dms.ui.shell.palette.model

/** A titled block of palette lines, such as "Documents" or "Go to". */
data class PaletteGroupModel(val title: String, val items: List<PaletteItemModel>)
