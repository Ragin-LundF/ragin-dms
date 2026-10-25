package dms.storage.catalog

import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.library.model.LibraryFolderModel
import dms.storage.catalog.dto.CatalogDto
import dms.storage.catalog.mapper.CatalogMapper
import dms.storage.fs.AtomicFiles
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import dms.storage.platform.SystemTrash
import dms.storage.yaml.YamlMappers
import java.nio.file.Files
import java.nio.file.Path

/**
 * Reads and writes `dms-catalog.yaml` in the root of one library folder.
 *
 * A file this version cannot read, because it is newer or broken, is never overwritten: saving fails
 * instead, so no device loses what another one wrote. The same holds for a file with a higher revision
 * than the change was based on.
 */
class FileCatalogRepository(private val trash: SystemTrash) : CatalogRepository {

    override suspend fun load(folder: LibraryFolderModel): CatalogLoad {
        val result = StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val file = root.resolve(CatalogFiles.FILE_NAME)
            if (!Files.exists(file)) {
                return@run StorageResult.Success(value = CatalogLoad.NotFound)
            }
            when (val main = readCatalog(file = file)) {
                is StorageResult.Failure -> main
                is StorageResult.Success -> {
                    val copies = CatalogFiles.conflictCopies(root = root)
                        .mapNotNull { (readCatalog(file = it) as? StorageResult.Success)?.value }
                    StorageResult.Success(value = CatalogLoad.Found(catalog = main.value, conflictCopies = copies))
                }
            }
        }
        return when (result) {
            is StorageResult.Success -> result.value
            is StorageResult.Failure -> CatalogLoad.Failed(failure = result)
        }
    }

    override suspend fun save(
        folder: LibraryFolderModel,
        catalog: CatalogModel,
        basedOn: CatalogRevisionModel,
    ): StorageResult<Unit> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val file = root.resolve(CatalogFiles.FILE_NAME)
            val previous = if (Files.exists(file)) readDto(file = file) else null
            if (previous is StorageResult.Failure) {
                return@run previous
            }
            val previousDto = (previous as? StorageResult.Success)?.value
            if (previousDto != null && previousDto.revision > basedOn.value) {
                return@run StorageIo.failure(reason = StorageFailure.CATALOG_CHANGED)
            }
            val dto = CatalogMapper.toDto(catalog = catalog, previous = previousDto)
            AtomicFiles.write(target = file, bytes = YamlMappers.yaml.writeValueAsBytes(dto))
            StorageResult.Success(value = Unit)
        }
    }

    /** Moves the conflict copies to the trash. Copies that cannot be read stay, so nothing unread is lost. */
    override suspend fun discardConflictCopies(folder: LibraryFolderModel): StorageResult<Unit> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val readable = CatalogFiles.conflictCopies(root = root)
                .filter { readCatalog(file = it) is StorageResult.Success }
            val failed = readable.filterNot { trash.moveToTrash(libraryRoot = root, entry = it) }
            if (failed.isEmpty()) {
                StorageResult.Success(value = Unit)
            } else {
                StorageIo.failure(reason = StorageFailure.IO_ERROR, detail = "Trash failed")
            }
        }
    }

    private fun readCatalog(file: Path): StorageResult<CatalogModel> {
        val dto = readDto(file = file)
        if (dto is StorageResult.Failure) {
            return dto
        }
        return try {
            StorageResult.Success(value = CatalogMapper.toDomain(dto = (dto as StorageResult.Success).value))
        } catch (exception: IllegalArgumentException) {
            StorageIo.failure(reason = StorageFailure.UNSUPPORTED_FORMAT, detail = exception.message)
        }
    }

    private fun readDto(file: Path): StorageResult<CatalogDto> {
        return StorageIo.catching {
            val tree = YamlMappers.yaml.readTree(Files.readAllBytes(file))
            val migrated = CatalogMigrations.migrate(tree = tree)
                ?: return@catching StorageIo.failure(reason = StorageFailure.UNSUPPORTED_FORMAT)
            StorageResult.Success(value = YamlMappers.yaml.treeToValue(migrated, CatalogDto::class.java))
        }
    }
}
