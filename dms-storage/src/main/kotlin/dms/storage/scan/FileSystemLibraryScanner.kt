package dms.storage.scan

import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.ScanListener
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.change.model.FileFactsModel
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.storage.fs.ContentHasher
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import dms.storage.scan.model.ListedFileModel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.job
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.nio.file.attribute.BasicFileAttributes

/**
 * Lists every file of one library folder with NFC paths. Symbolic links are not followed. The scan
 * cache of each folder lives in [cacheDirectory], outside the library.
 *
 * A scan first lists the folder, which is cheap, and then reads the files one by one, so the listener
 * knows the total. The cache is saved every [checkpointEvery] hashed files: a cancelled or crashed
 * first scan does not have to read those files again.
 */
class FileSystemLibraryScanner(
    private val cacheDirectory: Path,
    private val checkpointEvery: Int = DEFAULT_CHECKPOINT,
) : LibraryScanner {

    override suspend fun scan(
        folder: LibraryFolderModel,
        catalog: CatalogModel,
        listener: ScanListener,
    ): StorageResult<FolderSnapshotModel> {
        val job = currentCoroutineContext().job
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val cache = ScanCache.load(path = cacheDirectory.resolve("${folder.id.value}.yaml"))
            val known = catalog.documents.values.associateBy { it.path }
            val listed = mutableListOf<ListedFileModel>()
            val placeholders = mutableMapOf<RelativePathModel, FileFactsModel>()
            list(directory = root, segments = emptyList(), known = known, listed = listed, placeholders = placeholders)
            listener.counted(total = listed.size)
            val files = mutableMapOf<RelativePathModel, FileFactsModel>()
            var sinceCheckpoint = 0
            listed.forEachIndexed { index, file ->
                job.ensureActive()
                var hashed = false
                val facts = cache.facts(
                    path = file.path,
                    size = file.size,
                    modified = file.modified,
                    known = known[file.path],
                ) {
                    hashed = true
                    ContentHasher.hash(path = file.entry)
                }
                files[file.path] = facts
                if (hashed && ++sinceCheckpoint >= checkpointEvery) {
                    cache.checkpoint()
                    sinceCheckpoint = 0
                }
                listener.fileDone(done = index + 1, path = file.path, facts = facts, hashed = hashed)
            }
            // A downloaded file wins over a leftover placeholder of the same name.
            placeholders.forEach { (path, facts) -> files.putIfAbsent(path, facts) }
            cache.save(present = files.keys)
            StorageResult.Success(value = FolderSnapshotModel(folderId = folder.id, files = files))
        }
    }

    private fun list(
        directory: Path,
        segments: List<String>,
        known: Map<RelativePathModel, DocumentModel>,
        listed: MutableList<ListedFileModel>,
        placeholders: MutableMap<RelativePathModel, FileFactsModel>,
    ) {
        val entries = Files.newDirectoryStream(directory).use { stream -> stream.sortedBy { it.fileName.toString() } }
        entries.forEach { entry ->
            val name = PathResolver.nameOf(entry = entry)
            if (IgnoredEntries.isIgnored(name = name, atRoot = segments.isEmpty())) {
                return@forEach
            }
            val attributes = Files.readAttributes(entry, BasicFileAttributes::class.java, LinkOption.NOFOLLOW_LINKS)
            val placeholderName = CloudPlaceholders.realNameOf(name = name)
            when {
                attributes.isDirectory -> list(
                    directory = entry,
                    segments = segments + name,
                    known = known,
                    listed = listed,
                    placeholders = placeholders,
                )

                !attributes.isRegularFile -> Unit
                placeholderName != null -> {
                    val path = RelativePathModel.fromSegments(segments = segments + placeholderName)
                    placeholders[path] = FileFactsModel(size = known[path]?.size ?: 0, contentHash = null)
                }

                else -> listed += ListedFileModel(
                    path = RelativePathModel.fromSegments(segments = segments + name),
                    entry = entry,
                    size = attributes.size(),
                    modified = attributes.lastModifiedTime().toMillis(),
                )
            }
        }
    }

    private companion object {
        const val DEFAULT_CHECKPOINT = 100
    }
}
