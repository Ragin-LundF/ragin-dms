package dms.ui.document.model

/** The metadata fields as the user is typing them; [documentDate] is `YYYY-MM-DD` or empty. */
data class MetadataDraftModel(
    val title: String = "",
    val documentDate: String = "",
    val correspondent: String = "",
)
