package dms.ui.support

import androidx.compose.ui.draganddrop.DragAndDropEvent
import androidx.compose.ui.graphics.ImageBitmap
import dms.domain.library.FolderLocation
import dms.domain.library.RelativePath
import dms.ui.PlatformActions
import java.nio.file.Path

/** Answers file dialogs with prepared paths and records what it was asked to show. */
class FakePlatform : PlatformActions {

    var nextFolder: String? = null
    var nextFiles: List<String> = emptyList()
    val shown: MutableList<String> = mutableListOf()
    var decodedImages: Int = 0

    override val fileManagerName: String = "file manager"

    override suspend fun chooseFolder(title: String): String? {
        return nextFolder
    }

    override suspend fun chooseFiles(title: String): List<String> {
        return nextFiles
    }

    override fun locate(folder: FolderLocation, path: RelativePath): String {
        return Path.of(folder.value).resolve(path.value).toString()
    }

    override fun showInFileManager(location: String): Boolean {
        shown += location
        return true
    }

    override fun openFile(location: String): Boolean {
        shown += location
        return true
    }

    override fun decodeImage(bytes: ByteArray): ImageBitmap? {
        decodedImages += 1
        return ImageBitmap(width = 1, height = 1)
    }

    override fun carriesFiles(event: DragAndDropEvent): Boolean {
        return false
    }

    override fun droppedFiles(event: DragAndDropEvent): List<String> {
        return emptyList()
    }
}
