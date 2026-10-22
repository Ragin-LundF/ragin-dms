package dms.storage.support

import dms.domain.document.model.ContentHashModel
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.security.MessageDigest
import java.util.HexFormat

/** Library folders and files on a temporary directory, with synthetic names only. */
object TestFolders {

    fun folder(location: Path, id: String = "primary", role: FolderRole = FolderRole.PRIMARY): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = id),
            displayName = id,
            location = FolderLocationModel(value = location.toString()),
            role = role,
        )
    }

    fun write(file: Path, content: String): Path {
        Files.createDirectories(file.parent)
        Files.writeString(file, content)
        return file
    }

    fun read(file: Path): String {
        return Files.readString(file)
    }

    fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }

    fun hashOf(content: String): ContentHashModel {
        val digest = MessageDigest.getInstance("SHA-256").digest(content.toByteArray(StandardCharsets.UTF_8))
        return ContentHashModel(value = HexFormat.of().formatHex(digest))
    }

    /** Names of the entries directly inside [directory], as stored on disk. */
    fun namesIn(directory: Path): Set<String> {
        return Files.newDirectoryStream(directory).use { entries -> entries.map { it.fileName.toString() }.toSet() }
    }
}
