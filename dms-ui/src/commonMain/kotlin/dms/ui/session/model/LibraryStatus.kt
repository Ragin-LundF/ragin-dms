package dms.ui.session.model

import dms.application.port.storage.model.StorageFailure
import dms.domain.library.model.LibraryFolderIdModel

/** Whether a library is open, and why not. */
sealed interface LibraryStatus {

    data object Loading : LibraryStatus

    /** No primary folder is configured on this device yet. */
    data object NotConfigured : LibraryStatus

    /**
     * The library is open. [foreignFolders] hold another library and are ignored; [unreachableFolders]
     * could not be read when it was opened.
     */
    data class Open(
        val foreignFolders: Set<LibraryFolderIdModel>,
        val unreachableFolders: Set<LibraryFolderIdModel>,
    ) : LibraryStatus

    /**
     * The primary folder or its catalog could not be read or saved; nothing was opened. [detail] is
     * what the file system reported, if anything.
     */
    data class PrimaryFailed(val reason: StorageFailure, val detail: String? = null) : LibraryStatus

    /** Opening failed unexpectedly, for example because the settings file is unreadable. */
    data class Failed(val detail: String?) : LibraryStatus
}
