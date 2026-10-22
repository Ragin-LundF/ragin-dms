package dms.application.export.model

import dms.application.port.storage.model.StorageResult

sealed interface ExportOutcome {

    /** [count] documents were written to [location]. */
    data class Exported(val count: Int, val location: String) : ExportOutcome

    /** The scope holds no documents; nothing was written. */
    data object Empty : ExportOutcome

    /** The target lies inside a library folder; an export never writes into the library. */
    data object InsideLibrary : ExportOutcome

    data object LibraryNotOpen : ExportOutcome

    data class Failed(val failure: StorageResult.Failure) : ExportOutcome
}
