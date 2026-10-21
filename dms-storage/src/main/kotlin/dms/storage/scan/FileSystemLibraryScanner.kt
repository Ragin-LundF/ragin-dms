package dms.storage.scan

import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.change.FileFacts
import dms.domain.change.FolderSnapshot
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath
import dms.storage.fs.ContentHasher
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.nio.file.attribute.BasicFileAttributes

/**
 * Lists every file of one library folder with NFC paths. Symbolic links are not followed. The scan
 * cache of each folder lives in [cacheDirectory], outside the library.
 */
class FileSystemLibraryScanner(private val cacheDirectory: Path) : LibraryScanner {

    override suspend fun scan(folder: LibraryFolder, catalog: Catalog): StorageResult<FolderSnapshot> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val cache = ScanCache.load(path = cacheDirectory.resolve("${folder.id.value}.yaml"))
            val files = mutableMapOf<RelativePath, FileFacts>()
            walk(directory = root, segments = emptyList(), catalog = catalog, cache = cache, files = files)
            cache.save(present = files.keys)
            StorageResult.Success(value = FolderSnapshot(folderId = folder.id, files = files))
        }
    }

    private fun walk(
        directory: Path,
        segments: List<String>,
        catalog: Catalog,
        cache: ScanCache,
        files: MutableMap<RelativePath, FileFacts>,
    ) {
        val entries = Files.newDirectoryStream(directory).use { stream -> stream.toList() }
        val placeholders = mutableMapOf<RelativePath, FileFacts>()
        entries.forEach { entry ->
            val name = PathResolver.nameOf(entry = entry)
            if (IgnoredEntries.isIgnored(name = name, atRoot = segments.isEmpty())) {
                return@forEach
            }
            val attributes = Files.readAttributes(entry, BasicFileAttributes::class.java, LinkOption.NOFOLLOW_LINKS)
            val placeholderName = CloudPlaceholders.realNameOf(name = name)
            when {
                attributes.isDirectory -> walk(
                    directory = entry,
                    segments = segments + name,
                    catalog = catalog,
                    cache = cache,
                    files = files,
                )

                !attributes.isRegularFile -> Unit
                placeholderName != null -> {
                    val path = RelativePath.fromSegments(segments = segments + placeholderName)
                    val size = catalog.findByPath(path = path)?.size ?: 0
                    placeholders[path] = FileFacts(size = size, contentHash = null)
                }

                else -> {
                    val path = RelativePath.fromSegments(segments = segments + name)
                    files[path] = cache.facts(
                        path = path,
                        size = attributes.size(),
                        modified = attributes.lastModifiedTime().toMillis(),
                        known = catalog.findByPath(path = path),
                    ) { ContentHasher.hash(path = entry) }
                }
            }
        }
        // A downloaded file wins over a leftover placeholder of the same name.
        placeholders.forEach { (path, facts) -> files.putIfAbsent(path, facts) }
    }
}
