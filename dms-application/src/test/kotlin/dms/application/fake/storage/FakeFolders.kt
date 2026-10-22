package dms.application.fake.storage

import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.ScanListener
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.ExternalFileModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.change.model.FileFactsModel
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import java.security.MessageDigest
import java.time.LocalDate

/**
 * Every library folder of a test, in memory: file contents per path, empty folders, the catalog file
 * with its conflict copies, and `.dms-text`. Every call for a folder in [unreachable] fails.
 */
class FakeFolders : DocumentStorage, LibraryScanner, CatalogRepository, TextStore {

    val unreachable: MutableSet<LibraryFolderIdModel> = mutableSetOf()
    val trashed: MutableList<Pair<LibraryFolderIdModel, RelativePathModel>> = mutableListOf()

    private val files = mutableMapOf<LibraryFolderIdModel, MutableMap<RelativePathModel, String>>()
    private val directories = mutableMapOf<LibraryFolderIdModel, MutableSet<RelativePathModel>>()
    /** The catalog file of each folder. */
    val catalogs: MutableMap<LibraryFolderIdModel, CatalogModel> = mutableMapOf()

    /** Conflict copies of the catalog file, as a sync client creates them. */
    val conflictCopies: MutableMap<LibraryFolderIdModel, MutableList<CatalogModel>> = mutableMapOf()

    private val texts = mutableMapOf<LibraryFolderIdModel, MutableMap<DocumentIdModel, String>>()
    private val external = mutableMapOf<ExternalFileModel, String>()

    // --- Test setup and inspection

    fun put(folder: LibraryFolderModel, path: String, content: String) {
        filesOf(folderId = folder.id)[RelativePathModel.of(raw = path)] = content
    }

    fun content(folder: LibraryFolderModel, path: String): String? {
        return filesOf(folderId = folder.id)[RelativePathModel.of(raw = path)]
    }

    fun paths(folder: LibraryFolderModel): Set<String> {
        return filesOf(folderId = folder.id).keys.map { it.value }.toSet()
    }

    fun hasDirectory(folder: LibraryFolderModel, path: String): Boolean {
        return isDirectory(folderId = folder.id, path = RelativePathModel.of(raw = path))
    }

    fun textIn(folder: LibraryFolderModel, documentId: DocumentIdModel): String? {
        return texts[folder.id]?.get(key = documentId)
    }

    fun external(location: String, content: String): ExternalFileModel {
        val file = ExternalFileModel(location = location)
        external[file] = content
        return file
    }

    // --- DocumentStorage

    override suspend fun importFile(
        folder: LibraryFolderModel,
        source: ExternalFileModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        unreachableFailure(folder = folder)?.let { return it }
        val content = external[source] ?: return failure(reason = StorageFailure.NOT_FOUND)
        if (exists(folderId = folder.id, path = target)) {
            return failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        filesOf(folderId = folder.id)[target] = content
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun copyFile(
        from: LibraryFolderModel,
        to: LibraryFolderModel,
        path: RelativePathModel,
        replaceExisting: Boolean,
    ): StorageResult<FileFactsModel> {
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

    override suspend fun createFolder(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        if (filesOf(folderId = folder.id).containsKey(key = path)) {
            return failure(reason = StorageFailure.ALREADY_EXISTS)
        }
        directories.getOrPut(key = folder.id) { mutableSetOf() }.add(element = path)
        return StorageResult.Success(value = Unit)
    }

    override suspend fun move(
        folder: LibraryFolderModel,
        from: RelativePathModel,
        to: RelativePathModel,
    ): StorageResult<Unit> {
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

    override suspend fun replaceWith(
        folder: LibraryFolderModel,
        target: RelativePathModel,
        replacement: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        unreachableFailure(folder = folder)?.let { return it }
        val folderFiles = filesOf(folderId = folder.id)
        val content = folderFiles.remove(key = replacement) ?: return failure(reason = StorageFailure.NOT_FOUND)
        if (folderFiles.containsKey(key = target)) {
            trashed += folder.id to target
        }
        folderFiles[target] = content
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun moveToTrash(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<Unit> {
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

    override suspend fun readFacts(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<FileFactsModel> {
        unreachableFailure(folder = folder)?.let { return it }
        val content = filesOf(folderId = folder.id)[path] ?: return failure(reason = StorageFailure.NOT_FOUND)
        return StorageResult.Success(value = factsOf(content = content))
    }

    override suspend fun sameContent(
        folder: LibraryFolderModel,
        first: RelativePathModel,
        second: RelativePathModel,
    ): StorageResult<Boolean> {
        unreachableFailure(folder = folder)?.let { return it }
        val folderFiles = filesOf(folderId = folder.id)
        val firstContent = folderFiles[first] ?: return failure(reason = StorageFailure.NOT_FOUND)
        val secondContent = folderFiles[second] ?: return failure(reason = StorageFailure.NOT_FOUND)
        return StorageResult.Success(value = first != second && firstContent == secondContent)
    }

    /** Creation days the test set; files without one have none, like on file systems without them. */
    val creationDays: MutableMap<RelativePathModel, LocalDate> = mutableMapOf()

    override suspend fun createdOn(folder: LibraryFolderModel, path: RelativePathModel): LocalDate? {
        return creationDays[path]
    }

    override suspend fun listNames(folder: LibraryFolderModel, path: RelativePathModel): StorageResult<List<String>> {
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
        folder: LibraryFolderModel,
        catalog: CatalogModel,
        listener: ScanListener,
    ): StorageResult<FolderSnapshotModel> {
        unreachableFailure(folder = folder)?.let { return it }
        val facts = filesOf(folderId = folder.id).mapValues { (_, content) -> factsOf(content = content) }
        listener.counted(total = facts.size)
        facts.entries.forEachIndexed { index, (path, fileFacts) ->
            listener.fileDone(done = index + 1, path = path, facts = fileFacts, hashed = true)
        }
        return StorageResult.Success(value = FolderSnapshotModel(folderId = folder.id, files = facts))
    }

    // --- CatalogRepository

    override suspend fun load(folder: LibraryFolderModel): CatalogLoad {
        unreachableFailure(folder = folder)?.let { return CatalogLoad.Failed(failure = it) }
        val catalog = catalogs[folder.id] ?: return CatalogLoad.NotFound
        return CatalogLoad.Found(catalog = catalog, conflictCopies = conflictCopies[folder.id].orEmpty().toList())
    }

    override suspend fun save(folder: LibraryFolderModel, catalog: CatalogModel): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        catalogs[folder.id] = catalog
        return StorageResult.Success(value = Unit)
    }

    override suspend fun discardConflictCopies(folder: LibraryFolderModel): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        conflictCopies.remove(key = folder.id)
        return StorageResult.Success(value = Unit)
    }

    // --- TextStore

    override suspend fun write(
        folder: LibraryFolderModel,
        documentId: DocumentIdModel,
        text: String,
    ): StorageResult<Unit> {
        unreachableFailure(folder = folder)?.let { return it }
        texts.getOrPut(key = folder.id) { mutableMapOf() }[documentId] = text
        return StorageResult.Success(value = Unit)
    }

    override suspend fun read(folder: LibraryFolderModel, documentId: DocumentIdModel): String? {
        if (folder.id in unreachable) {
            return null
        }
        return textIn(folder = folder, documentId = documentId)
    }

    // --- Helpers

    private fun filesOf(folderId: LibraryFolderIdModel): MutableMap<RelativePathModel, String> {
        return files.getOrPut(key = folderId) { mutableMapOf() }
    }

    private fun exists(folderId: LibraryFolderIdModel, path: RelativePathModel): Boolean {
        return filesOf(folderId = folderId).containsKey(key = path) || isDirectory(folderId = folderId, path = path)
    }

    private fun isDirectory(folderId: LibraryFolderIdModel, path: RelativePathModel): Boolean {
        val explicit = directories[folderId].orEmpty()
        return path in explicit ||
            explicit.any { it.isInside(folder = path) } ||
            filesOf(folderId = folderId).keys.any { it.isInside(folder = path) }
    }

    private fun unreachableFailure(folder: LibraryFolderModel): StorageResult.Failure? {
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
        fun factsOf(content: String): FileFactsModel {
            val bytes = content.toByteArray()
            return FileFactsModel(size = bytes.size.toLong(), contentHash = hashOf(content = content))
        }

        fun hashOf(content: String): ContentHashModel {
            val digest = MessageDigest.getInstance("SHA-256").digest(content.toByteArray())
            return ContentHashModel(value = digest.joinToString(separator = "") { "%02x".format(it) })
        }
    }
}
