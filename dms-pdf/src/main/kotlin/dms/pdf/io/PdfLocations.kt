package dms.pdf.io

import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.nio.file.Path

/**
 * Where library files are on this device: [existing] finds a file that is there, [target] names the
 * place of a new one; both `null` while the folder is not reachable. The composition root passes the
 * storage adapter's mapping, so this module never resolves library paths itself.
 */
class PdfLocations(
    val existing: (LibraryFolderModel, RelativePathModel) -> Path?,
    val target: (LibraryFolderModel, RelativePathModel) -> Path?,
)
