package dms.storage.state

import dms.application.port.state.TaskStore
import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.task.model.TaskKeyModel
import dms.storage.state.dto.task.TasksDto
import dms.storage.yaml.YamlFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.nio.file.Path

/**
 * Pending suggestions and dismissed tasks in `tasks.yaml`. The conflicts of the last catalog merge are
 * kept in memory only: every opening of the library computes them again.
 */
class FileTaskStore(path: Path) : TaskStore {

    private val file = YamlFile(path = path, type = TasksDto::class.java)
    private val mutex = Mutex()
    private var mergeConflicts: List<CatalogMergeConflict> = emptyList()

    override suspend fun saveSuggestion(suggestion: ClassificationSuggestionModel) {
        update { tasks ->
            val others = tasks.suggestions.filterNot { it.documentId == suggestion.documentId.value }
            tasks.copy(suggestions = others + SuggestionMapper.toDto(suggestion = suggestion))
        }
    }

    override suspend fun suggestions(): List<ClassificationSuggestionModel> {
        return mutex.withLock { read() }.suggestions.map { SuggestionMapper.toDomain(dto = it) }
    }

    override suspend fun removeSuggestion(documentId: DocumentIdModel) {
        update { tasks -> tasks.copy(suggestions = tasks.suggestions.filterNot { it.documentId == documentId.value }) }
    }

    override suspend fun replaceMergeConflicts(conflicts: List<CatalogMergeConflict>) {
        mutex.withLock { mergeConflicts = conflicts }
    }

    override suspend fun mergeConflicts(): List<CatalogMergeConflict> {
        return mutex.withLock { mergeConflicts }
    }

    override suspend fun dismiss(key: TaskKeyModel) {
        update { tasks -> tasks.copy(dismissed = (tasks.dismissed + key.value).distinct().sorted()) }
    }

    override suspend fun dismissedKeys(): Set<TaskKeyModel> {
        return mutex.withLock { read() }.dismissed.map { TaskKeyModel(value = it) }.toSet()
    }

    private suspend fun update(change: (TasksDto) -> TasksDto) {
        mutex.withLock {
            val updated = change(read())
            withContext(context = Dispatchers.IO) { file.write(value = updated) }
        }
    }

    private suspend fun read(): TasksDto {
        return withContext(context = Dispatchers.IO) { file.read() ?: TasksDto() }
    }
}
