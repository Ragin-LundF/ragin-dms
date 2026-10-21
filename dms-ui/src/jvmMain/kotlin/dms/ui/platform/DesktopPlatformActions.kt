package dms.ui.platform

import dms.domain.library.FolderLocation
import dms.domain.library.RelativePath
import dms.ui.PlatformActions
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.swing.Swing
import kotlinx.coroutines.withContext
import java.awt.Desktop
import java.awt.FileDialog
import java.awt.Frame
import java.nio.file.Path
import javax.swing.JFileChooser

/** File dialogs and the file manager of macOS, Windows and Linux. */
class DesktopPlatformActions : PlatformActions {

    override val fileManagerName: String = when {
        HostOs.isMac -> "Finder"
        HostOs.isWindows -> "Explorer"
        else -> "file manager"
    }

    override suspend fun chooseFolder(title: String): String? {
        return withContext(context = Dispatchers.Swing) {
            if (HostOs.isMac) macFolderDialog(title = title) else folderChooser(title = title)
        }
    }

    override suspend fun chooseFiles(title: String): List<String> {
        return withContext(context = Dispatchers.Swing) {
            val dialog = FileDialog(null as Frame?, title, FileDialog.LOAD)
            dialog.isMultipleMode = true
            dialog.isVisible = true
            dialog.files.map { it.toPath().toString() }
        }
    }

    override fun locate(folder: FolderLocation, path: RelativePath): String {
        return Path.of(folder.value).resolve(path.value).toString()
    }

    override fun showInFileManager(location: String): Boolean {
        val command = when {
            HostOs.isMac -> listOf("open", "-R", location)
            HostOs.isWindows -> listOf("explorer.exe", "/select,$location")
            else -> listOf("xdg-open", Path.of(location).parent.toString())
        }
        return start(command = command)
    }

    override fun openFile(location: String): Boolean {
        val desktop = Desktop.isDesktopSupported() && Desktop.getDesktop().isSupported(Desktop.Action.OPEN)
        if (!desktop) {
            return start(command = listOf("xdg-open", location))
        }
        return runCatching { Desktop.getDesktop().open(Path.of(location).toFile()) }.isSuccess
    }

    /** macOS shows folders in its native dialog only with this property set. */
    private fun macFolderDialog(title: String): String? {
        System.setProperty(MAC_FOLDER_DIALOG, "true")
        try {
            val dialog = FileDialog(null as Frame?, title, FileDialog.LOAD)
            dialog.isVisible = true
            val directory = dialog.directory ?: return null
            val name = dialog.file ?: return null
            return Path.of(directory, name).toString()
        } finally {
            System.setProperty(MAC_FOLDER_DIALOG, "false")
        }
    }

    private fun folderChooser(title: String): String? {
        val chooser = JFileChooser()
        chooser.dialogTitle = title
        chooser.fileSelectionMode = JFileChooser.DIRECTORIES_ONLY
        if (chooser.showOpenDialog(null) != JFileChooser.APPROVE_OPTION) {
            return null
        }
        return chooser.selectedFile.toPath().toString()
    }

    private fun start(command: List<String>): Boolean {
        return runCatching { ProcessBuilder(command).start() }.isSuccess
    }

    private companion object {
        const val MAC_FOLDER_DIALOG = "apple.awt.fileDialogForDirectories"
    }
}
