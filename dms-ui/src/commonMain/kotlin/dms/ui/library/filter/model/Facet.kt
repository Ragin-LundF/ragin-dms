package dms.ui.library.filter.model

/**
 * A property the documents can be filtered by, shown as a chip above the documents. [label] names the
 * chip, [key] starts the filter's token in the search field, as in `from: Sample Utility`.
 */
enum class Facet(val label: String, val key: String) {
    CATEGORY(label = "Category", key = "category"),
    TAG(label = "Tag", key = "tag"),
    YEAR(label = "Year", key = "year"),
    CORRESPONDENT(label = "Correspondent", key = "from"),
    TYPE(label = "Type", key = "type"),
    FOLDER(label = "Folder", key = "folder"),
}
