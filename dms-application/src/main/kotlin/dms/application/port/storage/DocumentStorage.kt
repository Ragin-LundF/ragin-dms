package dms.application.port.storage

import dms.application.port.storage.model.ExternalFileModel
import dms.application.port.storage.model.FolderEntriesModel
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.time.LocalDate

/**
 * File operations inside one library folder. Implementations never overwrite or delete a file unless
 * the call says so explicitly, write atomically, and create missing parent folders of a target path.
 */
interface DocumentStorage {

    /** Copies [source] into [folder] at [target] and returns the facts of the written file (hashed). */
    suspend fun importFile(
        folder: LibraryFolderModel,
        source: ExternalFileModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /**
     * Copies [path] from one library folder to another, verified by hash. With [replaceExisting], an
     * existing file at the target is moved to the system trash first, so a replaced version is never lost.
     */
    suspend fun copyFile(
        from: LibraryFolderModel,
        to: LibraryFolderModel,
        path: RelativePathModel,
        replaceExisting: Boolean,
    ): StorageResult<FileFactsModel>

    suspend fun createFolder(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit>

    /** Moves or renames a file or folder. A rename that only changes the case must work as well. */
    suspend fun move(folder: LibraryFolderModel, from: RelativePathModel, to: RelativePathModel): StorageResult<Unit>

    /**
     * Puts [replacement] in place of [target] in one folder: the old file goes to the system trash,
     * so a replaced version is never lost, and [replacement] is moved to [target]. Used for a new
     * version of a file, such as an edited PDF.
     */
    suspend fun replaceWith(
        folder: LibraryFolderModel,
        target: RelativePathModel,
        replacement: RelativePathModel,
    ): StorageResult<FileFactsModel>

    /** Moves a file or folder to the system trash. */
    suspend fun moveToTrash(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit>

    /** Size and content hash of one file; always hashed. */
    suspend fun readFacts(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<FileFactsModel>

    /**
     * Whether [first] and [second] in [folder] are two files with exactly the same bytes, compared
     * byte by byte. Two names of one file, as on a file system that ignores case, are not two files.
     */
    suspend fun sameContent(
        folder: LibraryFolderModel,
        first: RelativePathModel,
        second: RelativePathModel,
    ): StorageResult<Boolean>

    /**
     * The day the file was created, as far as the file system knows it, or `null` when it does not
     * keep creation times or the file is missing.
     */
    suspend fun createdOn(folder: LibraryFolderModel, path: RelativePathModel): LocalDate?

    /** Names of the entries directly inside [path], for collision checks. */
    suspend fun listNames(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<List<String>>

    /**
     * Every folder and file below [path], at any depth, as paths from the root; the tool's own files and
     * system clutter are left out, as in a scan. `NOT_FOUND` when [path] is no folder.
     */
    suspend fun listEntries(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<FolderEntriesModel>

    /**
     * Every folder below the top level of [folder], at any depth, also empty ones; files are not read. Left
     * out as in a scan: the tool's own folders and system clutter.
     */
    suspend fun listFolders(folder: LibraryFolderModel): StorageResult<Set<RelativePathModel>>
}
