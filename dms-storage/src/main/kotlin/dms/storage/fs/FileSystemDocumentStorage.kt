package dms.storage.fs

import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.ExternalFileModel
import dms.application.port.storage.model.FolderEntriesModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.storage.platform.SystemTrash
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.BasicFileAttributes
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

/**
 * File operations inside one library folder. Never overwrites: an existing entry with the same name,
 * ignoring case, is a collision, and a replaced file goes to the trash first.
 */
class FileSystemDocumentStorage(private val trash: SystemTrash) : DocumentStorage {

    override suspend fun importFile(
        folder: LibraryFolderModel,
        source: ExternalFileModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return inFolder(folder = folder) { root ->
            val sourcePath = Path.of(source.location)
            if (!Files.isRegularFile(sourcePath)) {
                return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            }
            val targetPath = PathResolver.target(root = root, path = target)
            if (PathResolver.clashing(directory = targetPath.parent, name = target.fileName) != null) {
                return@inFolder StorageIo.failure(reason = StorageFailure.ALREADY_EXISTS)
            }
            AtomicFiles.copy(source = sourcePath, target = targetPath)
        }
    }

    override suspend fun copyFile(
        from: LibraryFolderModel,
        to: LibraryFolderModel,
        path: RelativePathModel,
        replaceExisting: Boolean,
    ): StorageResult<FileFactsModel> {
        return inFolder(folder = from) { fromRoot ->
            val toRoot = PathResolver.root(folder = to)
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val source = PathResolver.existing(root = fromRoot, path = path)?.takeIf { Files.isRegularFile(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val target = PathResolver.target(root = toRoot, path = path)
            val existing = PathResolver.clashing(directory = target.parent, name = path.fileName)
            if (existing != null) {
                if (!replaceExisting) {
                    return@inFolder StorageIo.failure(reason = StorageFailure.ALREADY_EXISTS)
                }
                if (!trash.moveToTrash(libraryRoot = toRoot, entry = existing)) {
                    return@inFolder StorageIo.failure(reason = StorageFailure.IO_ERROR, detail = "Trash failed")
                }
            }
            AtomicFiles.copy(source = source, target = target)
        }
    }

    override suspend fun createFolder(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit> {
        return inFolder(folder = folder) { root ->
            val target = PathResolver.target(root = root, path = path)
            val existing = PathResolver.clashing(directory = target.parent, name = path.fileName)
            when {
                existing == null -> Files.createDirectories(target)
                !Files.isDirectory(existing) || PathResolver.nameOf(entry = existing) != path.fileName ->
                    return@inFolder StorageIo.failure(reason = StorageFailure.ALREADY_EXISTS)
            }
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun move(
        folder: LibraryFolderModel,
        from: RelativePathModel,
        to: RelativePathModel,
    ): StorageResult<Unit> {
        return inFolder(folder = folder) { root ->
            val source = PathResolver.existing(root = root, path = from)
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val target = PathResolver.target(root = root, path = to)
            val clash = PathResolver.clashing(directory = target.parent, name = to.fileName)
            val caseOnly = clash != null && Files.isSameFile(clash, source)
            if (clash != null && !caseOnly) {
                return@inFolder StorageIo.failure(reason = StorageFailure.ALREADY_EXISTS)
            }
            Files.createDirectories(target.parent)
            if (caseOnly) {
                // A case-insensitive file system treats both names as the same file; go through a third name.
                val intermediate = AtomicFiles.tempFileFor(target = source)
                Files.move(source, intermediate)
                Files.move(intermediate, target)
            } else {
                Files.move(source, target)
            }
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun replaceWith(
        folder: LibraryFolderModel,
        target: RelativePathModel,
        replacement: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return inFolder(folder = folder) { root ->
            val source = PathResolver.existing(root = root, path = replacement)
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val existing = PathResolver.existing(root = root, path = target)
            if (existing != null && !trash.moveToTrash(libraryRoot = root, entry = existing)) {
                return@inFolder StorageIo.failure(reason = StorageFailure.IO_ERROR, detail = "Trash failed")
            }
            val destination = existing ?: PathResolver.target(root = root, path = target)
            Files.createDirectories(destination.parent)
            Files.move(source, destination, StandardCopyOption.ATOMIC_MOVE)
            val facts = FileFactsModel(
                size = Files.size(destination),
                contentHash = ContentHasher.hash(path = destination),
            )
            StorageResult.Success(value = facts)
        }
    }

    override suspend fun moveToTrash(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit> {
        return inFolder(folder = folder) { root ->
            val entry = PathResolver.existing(root = root, path = path)
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            if (!trash.moveToTrash(libraryRoot = root, entry = entry)) {
                return@inFolder StorageIo.failure(reason = StorageFailure.IO_ERROR, detail = "Trash failed")
            }
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun readFacts(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<FileFactsModel> {
        return inFolder(folder = folder) { root ->
            val file = PathResolver.existing(root = root, path = path)?.takeIf { Files.isRegularFile(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val facts = FileFactsModel(size = Files.size(file), contentHash = ContentHasher.hash(path = file))
            StorageResult.Success(value = facts)
        }
    }

    override suspend fun sameContent(
        folder: LibraryFolderModel,
        first: RelativePathModel,
        second: RelativePathModel,
    ): StorageResult<Boolean> {
        return inFolder(folder = folder) { root ->
            val firstFile = PathResolver.existing(root = root, path = first)?.takeIf { Files.isRegularFile(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val secondFile = PathResolver.existing(root = root, path = second)?.takeIf { Files.isRegularFile(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            // `mismatch` reads both files and returns -1 only if every byte, and so the size, is the same.
            val same = !Files.isSameFile(firstFile, secondFile) && Files.mismatch(firstFile, secondFile) == -1L
            StorageResult.Success(value = same)
        }
    }

    override suspend fun createdOn(folder: LibraryFolderModel, path: RelativePathModel): LocalDate? {
        val result = inFolder(folder = folder) { root ->
            val file = PathResolver.existing(root = root, path = path)?.takeIf { Files.isRegularFile(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            // File systems without creation times report the modification time, or the epoch.
            val created = Files.readAttributes(file, BasicFileAttributes::class.java).creationTime().toInstant()
            val day = created.takeIf { it != Instant.EPOCH }?.atZone(ZoneId.systemDefault())?.toLocalDate()
            StorageResult.Success(value = day)
        }
        return (result as? StorageResult.Success)?.value
    }

    override suspend fun listNames(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<List<String>> {
        return inFolder(folder = folder) { root ->
            val directory = PathResolver.existing(root = root, path = path)?.takeIf { Files.isDirectory(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            val names = Files.newDirectoryStream(directory).use { entries ->
                entries.map { PathResolver.nameOf(entry = it) }
            }
            StorageResult.Success(value = names)
        }
    }

    override suspend fun listEntries(
        folder: LibraryFolderModel,
        path: RelativePathModel,
    ): StorageResult<FolderEntriesModel> {
        return inFolder(folder = folder) { root ->
            val directory = PathResolver.existing(root = root, path = path)?.takeIf { Files.isDirectory(it) }
                ?: return@inFolder StorageIo.failure(reason = StorageFailure.NOT_FOUND)
            StorageResult.Success(value = FolderWalk.entries(root = root, directory = directory))
        }
    }

    override suspend fun listFolders(folder: LibraryFolderModel): StorageResult<Set<RelativePathModel>> {
        return inFolder(folder = folder) { root -> StorageResult.Success(value = FolderWalk.folders(root = root)) }
    }

    private suspend fun <T> inFolder(
        folder: LibraryFolderModel,
        block: (Path) -> StorageResult<T>,
    ): StorageResult<T> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
            if (root == null) {
                StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            } else {
                block(root)
            }
        }
    }
}
