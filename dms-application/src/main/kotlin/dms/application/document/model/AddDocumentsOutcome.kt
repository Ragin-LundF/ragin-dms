package dms.application.document.model

sealed interface AddDocumentsOutcome {

    /** One entry per file, in the given order, each with its own outcome. */
    data class Added(val files: List<AddedFileModel>) : AddDocumentsOutcome

    data object LibraryNotOpen : AddDocumentsOutcome
}
