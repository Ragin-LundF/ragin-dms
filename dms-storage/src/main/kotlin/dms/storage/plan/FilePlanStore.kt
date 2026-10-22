package dms.storage.plan

import dms.application.port.plan.PlanStore
import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.model.PlannedMoveModel
import dms.storage.fs.AtomicFiles
import dms.storage.fs.PathResolver
import dms.storage.fs.StorageIo
import dms.storage.plan.dto.PlannedMoveDto
import dms.storage.plan.dto.RestructureRecordDto
import dms.storage.yaml.YamlMappers
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant

/** Applied restructures in `.dms-plans/<name>.yaml` of one library folder. A broken file reads as none. */
class FilePlanStore : PlanStore {

    override suspend fun write(folder: LibraryFolderModel, record: RestructureRecordModel): StorageResult<Unit> {
        return StorageIo.run {
            val root = PathResolver.root(folder = folder)
                ?: return@run StorageIo.failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val dto = RestructureRecordDto(
                appliedAt = record.appliedAt.toString(),
                template = record.template,
                moves = record.moves.map {
                    PlannedMoveDto(documentId = it.documentId.value, from = it.from.value, to = it.to.value)
                },
            )
            AtomicFiles.write(
                target = fileOf(root = root, name = record.name),
                bytes = YamlMappers.yaml.writeValueAsBytes(dto),
            )
            StorageResult.Success(value = Unit)
        }
    }

    override suspend fun read(folder: LibraryFolderModel, name: String): RestructureRecordModel? {
        return withContext(context = Dispatchers.IO) {
            val file = PathResolver.root(folder = folder)?.let { fileOf(root = it, name = name) }
            runCatching {
                file?.takeIf { Files.isRegularFile(it) }?.let { toRecord(name = name, file = it) }
            }.getOrNull()
        }
    }

    override suspend fun names(folder: LibraryFolderModel): List<String> {
        return withContext(context = Dispatchers.IO) {
            val directory = PathResolver.root(folder = folder)?.resolve(PLANS_FOLDER)
            if (directory == null || !Files.isDirectory(directory)) {
                return@withContext emptyList()
            }
            Files.newDirectoryStream(directory, "*$EXTENSION").use { entries ->
                entries.map { it.fileName.toString().removeSuffix(suffix = EXTENSION) }.sortedDescending()
            }
        }
    }

    private fun toRecord(name: String, file: Path): RestructureRecordModel {
        val dto = YamlMappers.yaml.readValue(Files.readAllBytes(file), RestructureRecordDto::class.java)
        return RestructureRecordModel(
            name = name,
            appliedAt = Instant.parse(dto.appliedAt),
            template = dto.template,
            moves = dto.moves.map {
                PlannedMoveModel(
                    documentId = DocumentIdModel(value = it.documentId),
                    from = RelativePathModel.of(raw = it.from),
                    to = RelativePathModel.of(raw = it.to),
                )
            },
        )
    }

    private fun fileOf(root: Path, name: String): Path {
        require(value = NAME.matches(input = name)) { "Not a plan name: $name" }
        return root.resolve(PLANS_FOLDER).resolve("$name$EXTENSION")
    }

    companion object {
        const val PLANS_FOLDER: String = ".dms-plans"
        private const val EXTENSION = ".yaml"
        private val NAME = Regex(pattern = "^[A-Za-z0-9_-]+$")
    }
}
