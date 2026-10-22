package dms.application.export.model

/**
 * How an export is laid out: [flatten] puts every file in one folder (clashing names get a suffix),
 * [renameToTitle] names files after their title, [index] adds `index.csv` and `index.html`, [zip]
 * writes one ZIP file instead of a folder.
 */
data class ExportOptionsModel(
    val flatten: Boolean = false,
    val renameToTitle: Boolean = false,
    val index: Boolean = true,
    val zip: Boolean = false,
)
