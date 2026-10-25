package dms.ui.library.documents.model

/** What the documents are grouped by, each group under a header that stays in place while it scrolls. */
enum class GroupKey(val label: String) {
    MONTH(label = "By month"),
    CATEGORY(label = "By category"),
    SENDER(label = "By sender"),
}
