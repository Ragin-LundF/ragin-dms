package dms.ui

import androidx.compose.ui.draganddrop.DragAndDropEvent
import androidx.compose.ui.graphics.ImageBitmap
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.RelativePathModel

/** What the UI needs from the operating system. Implemented in `jvmMain`, faked in tests. */
interface PlatformActions {

    /** "Finder", "Explorer" or "file manager", for labels such as "Show in Finder". */
    val fileManagerName: String

    /** Asks the user for a folder; returns its absolute path, or `null` if the user cancelled. */
    suspend fun chooseFolder(title: String): String?

    /** Asks the user for files to add; returns their absolute paths, empty if the user cancelled. */
    suspend fun chooseFiles(title: String): List<String>

    /** The absolute location of [path] inside the library folder at [folder]. */
    fun locate(folder: FolderLocationModel, path: RelativePathModel): String

    /** Returns `false` if the file manager could not be opened. */
    fun showInFileManager(location: String): Boolean

    /** Opens a file in its default application; returns `false` if that failed. */
    fun openFile(location: String): Boolean

    /** Decodes a JPEG or PNG for the screen; `null` if [bytes] are no image. */
    fun decodeImage(bytes: ByteArray): ImageBitmap?

    /** Whether something dragged in from outside the app carries files. */
    fun carriesFiles(event: DragAndDropEvent): Boolean

    /** The absolute paths of the regular files dropped with [event]; folders are left out. */
    fun droppedFiles(event: DragAndDropEvent): List<String>
}
