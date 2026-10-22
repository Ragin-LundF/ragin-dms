package dms.application.port.export.model

/**
 * What an export writes: the files and, if wanted, an index (`index.csv` and `index.html`) whose
 * columns are the fixed ones plus [fieldLabels], in the order of [IndexRowModel.fields].
 */
data class ExportBundleModel(
    val entries: List<ExportEntryModel>,
    val index: List<IndexRowModel>?,
    val fieldLabels: List<String> = emptyList(),
)
