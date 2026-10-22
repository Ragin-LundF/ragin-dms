package dms.application.port.export.model

/** One line of the bundle's index: where the file is in the bundle and what the catalog knows of it. */
data class IndexRowModel(
    val path: String,
    val title: String,
    val date: String,
    val correspondent: String,
    val category: String,
    val tags: String,
    val fields: List<String>,
    val note: String,
)
