package dms.application.library.model.outcome

import dms.application.library.model.ScanResultModel

sealed interface ScanOutcome {

    data class Scanned(val result: ScanResultModel) : ScanOutcome

    /** No library is open, or it was closed while the scan ran. */
    data object LibraryNotOpen : ScanOutcome
}
