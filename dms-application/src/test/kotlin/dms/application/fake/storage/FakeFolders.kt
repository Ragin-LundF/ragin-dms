package dms.application.fake.storage

import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.ScanListener
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.ExternalFile
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.change.FileFacts
import dms.domain.change.FolderSnapshot
import dms.domain.document.ContentHash
import dms.domain.document.DocumentId
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import java.security.MessageDigest
import java.time.LocalDate

/**
 * Every library folder of a test, in memory: file contents per path, empty folders, the catalog file
 * with its conflict copies, and `.dms-text`. Every call for a folder in [unreachable] fails.
 */
class FakeFolders : DocumentStorage, LibraryScanner, CatalogRepository, TextStore {

    val unreachable: MutableSet<LibraryFolderId> = mutableSetOf()
    val trashed: MutableList<Pair<LibraryFolderId, RelativePath>> = mutableListOf()

    private val files = mutableMapOf<LibraryFolderId, MutableMap<RelativePath, String>>()
    private val directories = mutableMapOf<LibraryFolderId, MutableSet<RelativePath>>()
    /** The catalog file of each folder. */
    val catalogs: MutableMap<LibraryFolderId, Catalog> = mutableMapOf()

    /** Conflict copies of the catalog file, as a sync client creates them. */
    val conflictCopies: MutableMap<LibraryFolderId, MutableList<Catalog>> = mutableMapOf()

    private val texts = mutableMapOf<LibraryFolderId, MutableMap<DocumentId, String>>()
    private val external = mutableMapOf<ExternalFile, String>()

    // --- Test setup and inspection

    fun put(folder: LibraryFolder, path: String, content: String) {
        filesOf(folderId = folder.id)[RelativePath.of(raw = path)] = content
    }

    fun content(folder: LibraryFolder, path: String): String? {
        return filesOf(folderId = folder.id)[RelativePath.of(raw = path)]
    }

    fun paths(folder: LibraryFolder): Set<String> {
        return filesOf(folderId = folder.id).keys.map { it.value }.toSet()
    }

    fun hasDirectory(folder: LibraryFolder, path: String): Boolean {
        return isDirectory(folderId = folder.id, path = RelativePath.of(raw = path))
    }

    fun textIn(folder: LibraryFolder, documentId: DocumentId): String? {
        return texts[folder.id]?.get(key = documentId)
    }

    fun external(location: String, content: String): ExternalFile {
        val file = ExternalFile(location = location)
        external[file] = content
        return file
    }

    // --- DocumentStorage

    override suspend fun importFile(
        folder: LibraryFolder,
        source: ExternalFile,
        target: RelativePath,
    ): StorageResult<FileFacts> {
        unreachableFailure(folder = folder)?.let { return it }
        val content = external[source] ?: return failure(reason = StorageFailure.NOT_FOUND)
        if (exists(folderId = folder.id, path = target)) {
            return failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        filesOf(folderId = folder.id)[target] = content
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun copyFile(
        from: LibraryFolder,
        to: LibraryFolder,
        path: RelativePath,
        replaceExisting: Boolean,
    ): StorageResult<FileFacts> {
        unreachableFailure(folder = from)?.let { return it }
        unreachableFailure(folder = to)?.let { return it }
        val content = filesOf(folderId = from.id)[path] ?: return failure(reason = StorageFailure.NOT_FOUND)
        if (filesOf(folderId = to.id).containsKey(key = path)) {
            if (!replaceExisting) {
                return failure(reason = StorageFailure.ALREADY_EXISTS)
            }
            trashed += to.id to path
        }
        filesOf(folderId = to.id)[path] = content
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun createFolder(folder: LibraryFolder, path: RelativePath): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        if (filesOf(folderId = folder.id).containsKey(key = path)) {
            return failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        directories.getOrPut(key = folder.id) { mutableSetOf() }.add(element = path)
        return StorageResult.Success(value = Unit)
    }

    override suspend fun move(folder: LibraryFolder, from: RelativePath, to: RelativePath): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        val folderFiles = filesOf(folderId = folder.id)
        val caseOnly = from.parent == to.parent && from.fileName.equals(other = to.fileName, ignoreCase = true)
        if (exists(folderId = folder.id, path = to) && !caseOnly) {
            return failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        val content = folderFiles.remove(key = from)
        if (content != null) {
            folderFiles[to] = content
            return StorageResult.Success(value = Unit)
        }
        if (!isDirectory(folderId = folder.id, path = from)) {
            return failure(reason = StorageFailure.NOT_FOUND)
        }
        folderFiles.keys.filter { it.isInside(folder = from) }.forEach { path ->
            folderFiles[path.relocate(from = from, to = to)] = folderFiles.getValue(key = path)
            folderFiles.remove(key = path)
        }
        directories[folder.id]?.let { dirs ->
            val moved = dirs.filter { it == from || it.isInside(folder = from) }
            dirs.removeAll(elements = moved.toSet())
            dirs.addAll(elements = moved.map { it.relocate(from = from, to = to) })
        }
        return StorageResult.Success(value = Unit)
    }

    override suspend fun moveToTrash(folder: LibraryFolder, path: RelativePath): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        val folderFiles = filesOf(folderId = folder.id)
        if (folderFiles.remove(key = path) != null) {
            trashed += folder.id to path
            return StorageResult.Success(value = Unit)
        }
        if (!isDirectory(folderId = folder.id, path = path)) {
            return failure(reason = StorageFailure.NOT_FOUND)
        }
        folderFiles.keys.removeAll { it.isInside(folder = path) }
        directories[folder.id]?.removeAll { it == path || it.isInside(folder = path) }
        trashed += folder.id to path
        return StorageResult.Success(value = Unit)
    }

    override suspend fun readFacts(folder: LibraryFolder, path: RelativePath): StorageResult<FileFacts> {
        unreachableFailure(folder = folder)?.let { return it }
        val content = filesOf(folderId = folder.id)[path] ?: return failure(reason = StorageFailure.NOT_FOUND)
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun sameContent(
        folder: LibraryFolder,
        first: RelativePath,
        second: RelativePath,
    ): StorageResult<Boolean> {
        unreachableFailure(folder = folder)?.let { return it }
        val folderFiles = filesOf(folderId = folder.id)
        val firstContent = folderFiles[first] ?: return failure(reason = StorageFailure.NOT_FOUND)
        val secondContent = folderFiles[second] ?: return failure(reason = StorageFailure.NOT_FOUND)
        return StorageResult.Success(value = first != second && firstContent == secondContent)
    }

    /** Creation days the test set; files without one have none, like on file systems without them. */
    val creationDays: MutableMap<RelativePath, LocalDate> = mutableMapOf()

    override suspend fun createdOn(folder: LibraryFolder, path: RelativePath): LocalDate? {
        return creationDays[path]
    }

    override suspend fun listNames(folder: LibraryFolder, path: RelativePath): StorageResult<List<String>> {
        unreachableFailure(folder = folder)?.let { return it }
        if (!path.isRoot && !isDirectory(folderId = folder.id, path = path)) {
            return failure(reason = StorageFailure.NOT_FOUND)
        }
        val depth = path.segments.size
        val entries = filesOf(folderId = folder.id).keys + directories[folder.id].orEmpty()
        val names = entries.filter { it.isInside(folder = path) }.map { it.segments[depth] }.distinct()
        return StorageResult.Success(value = names)
    }

    // --- LibraryScanner

    override suspend fun scan(
        folder: LibraryFolder,
        catalog: Catalog,
        listener: ScanListener,
    ): StorageResult<FolderSnapshot> {
        unreachableFailure(folder = folder)?.let { return it }
        val facts = filesOf(folderId = folder.id).mapValues { (_, content) -> factsOf(content = content) }
        listener.counted(total = facts.size)
        facts.entries.forEachIndexed { index, (path, fileFacts) ->
            listener.fileDone(done = index + 1, path = path, facts = fileFacts, hashed = true)
        }
        return StorageResult.Success(value = FolderSnapshot(folderId = folder.id, files = facts))
    }

    // --- CatalogRepository

    override suspend fun load(folder: LibraryFolder): CatalogLoad {
        unreachableFailure(folder = folder)?.let { return CatalogLoad.Failed(failure = it) }
        val catalog = catalogs[folder.id] ?: return CatalogLoad.NotFound
        return CatalogLoad.Found(catalog = catalog, conflictCopies = conflictCopies[folder.id].orEmpty().toList())
    }

    override suspend fun save(folder: LibraryFolder, catalog: Catalog): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        catalogs[folder.id] = catalog
        return StorageResult.Success(value = Unit)
    }

    override suspend fun discardConflictCopies(folder: LibraryFolder): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        conflictCopies.remove(key = folder.id)
        return StorageResult.Success(value = Unit)
    }

    // --- TextStore

    override suspend fun write(folder: LibraryFolder, documentId: DocumentId, text: String): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        texts.getOrPut(key = folder.id) { mutableMapOf() }[documentId] = text
        return StorageResult.Success(value = Unit)
    }

    override suspend fun read(folder: LibraryFolder, documentId: DocumentId): String? {
        if (folder.id in unreachable) {
            return null
        }
        return textIn(folder = folder, documentId = documentId)
    }

    // --- Helpers

    private fun filesOf(folderId: LibraryFolderId): MutableMap<RelativePath, String> {
        return files.getOrPut(key = folderId) { mutableMapOf() }
    }

    private fun exists(folderId: LibraryFolderId, path: RelativePath): Boolean {
        return filesOf(folderId = folderId).containsKey(key = path) || isDirectory(folderId = folderId, path = path)
    }

    private fun isDirectory(folderId: LibraryFolderId, path: RelativePath): Boolean {
        val explicit = directories[folderId].orEmpty()
        return path in explicit ||
            explicit.any { it.isInside(folder = path) } ||
            filesOf(folderId = folderId).keys.any { it.isInside(folder = path) }
    }

    private fun unreachableFailure(folder: LibraryFolder): StorageResult.Failure? {
        if (folder.id in unreachable) {
            return failure(reason = StorageFailure.FOLDER_UNREACHABLE)
        }
        return null
    }

    private fun failure(reason: StorageFailure): StorageResult.Failure {
        return StorageResult.Failure(reason = reason)
    }

    companion object {

        /** The facts a scanner reports for [content]: its length and SHA-256. */
        fun factsOf(content: String): FileFacts {
            val bytes = content.toByteArray()
            return FileFacts(size = bytes.size.toLong(), contentHash = hashOf(content = content))
        }

        fun hashOf(content: String): ContentHash {
            val digest = MessageDigest.getInstance("SHA-256").digest(content.toByteArray())
            return ContentHash(value = digest.joinToString(separator = "") { "%02x".format(it) })
        }
    }
}
