package dms.application.port.storage

import dms.application.port.storage.model.ExternalFile
import dms.application.port.storage.model.StorageResult
import dms.domain.change.FileFacts
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath
import java.time.LocalDate

/**
 * File operations inside one library folder. Implementations never overwrite or delete a file unless
 * the call says so explicitly, write atomically, and create missing parent folders of a target path.
 */
interface DocumentStorage {

    /** Copies [source] into [folder] at [target] and returns the facts of the written file (hashed). */
    suspend fun importFile(folder: LibraryFolder, source: ExternalFile, target: RelativePath): StorageResult<FileFacts>

    /**
     * Copies [path] from one library folder to another, verified by hash. With [replaceExisting], an
     * existing file at the target is moved to the system trash first, so a replaced version is never lost.
     */
    suspend fun copyFile(
        from: LibraryFolder,
        to: LibraryFolder,
        path: RelativePath,
        replaceExisting: Boolean,
    ): StorageResult<FileFacts>

    suspend fun createFolder(folder: LibraryFolder, path: RelativePath): StorageResult<Unit>

    /** Moves or renames a file or folder. A rename that only changes the case must work as well. */
    suspend fun move(folder: LibraryFolder, from: RelativePath, to: RelativePath): StorageResult<Unit>

    /** Moves a file or folder to the system trash. */
    suspend fun moveToTrash(folder: LibraryFolder, path: RelativePath): StorageResult<Unit>

    /** Size and content hash of one file; always hashed. */
    suspend fun readFacts(folder: LibraryFolder, path: RelativePath): StorageResult<FileFacts>

    /**
     * Whether [first] and [second] in [folder] are two files with exactly the same bytes, compared
     * byte by byte. Two names of one file, as on a file system that ignores case, are not two files.
     */
    suspend fun sameContent(folder: LibraryFolder, first: RelativePath, second: RelativePath): StorageResult<Boolean>

    /**
     * The day the file was created, as far as the file system knows it, or `null` when it does not
     * keep creation times or the file is missing.
     */
    suspend fun createdOn(folder: LibraryFolder, path: RelativePath): LocalDate?

    /** Names of the entries directly inside [path], for collision checks. */
    suspend fun listNames(folder: LibraryFolder, path: RelativePath): StorageResult<List<String>>
}
