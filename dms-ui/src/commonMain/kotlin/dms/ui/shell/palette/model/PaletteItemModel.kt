package dms.ui.shell.palette.model

import dms.domain.document.model.DocumentIdModel
import dms.ui.library.model.LibrarySelection

/** One line of the palette and what choosing it does. */
sealed interface PaletteItemModel {

    val label: String

    /** Opens the document in the library; [detail] says where it was found. */
    data class OpenDocument(val documentId: DocumentIdModel, override val label: String, val detail: String) :
        PaletteItemModel

    /** Shows the documents of [selection] in the library; [count] documents, if known. */
    data class GoTo(val selection: LibrarySelection, override val label: String, val kind: String, val count: Int?) :
        PaletteItemModel

    /** Runs [action]; while its lane is busy, [disabledReason] says why it waits, and choosing it runs nothing. */
    data class Run(val action: PaletteAction, val disabledReason: String? = null) : PaletteItemModel {
        override val label: String
            get() {
                return action.label
            }
    }

    /** Asks the whole library [question] on the Ask screen. */
    data class AskLibrary(val question: String) : PaletteItemModel {
        override val label: String
            get() {
                return "Ask the library: $question"
            }
    }
}
