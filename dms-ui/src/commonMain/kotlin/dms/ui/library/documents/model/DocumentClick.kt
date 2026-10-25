package dms.ui.library.documents.model

/** How a document was clicked: plainly, with ⌘/Ctrl to tick it, or with Shift to tick a range. */
enum class DocumentClick {
    PLAIN,
    TOGGLE,
    RANGE,
}
