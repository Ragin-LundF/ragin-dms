package dms.application.classification

import dms.application.classification.model.SuggestOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.DocumentContentReader
import dms.application.port.ai.model.ClassificationRequestModel
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
import dms.application.port.state.TaskStore
import dms.application.port.storage.TextStore
import dms.application.port.storage.model.FolderOperation
import dms.application.port.system.Clock
import dms.application.sync.Replicator
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.outline.model.FolderOutlineModel
import dms.domain.rule.match.RuleMatcher
import dms.domain.rule.match.model.RuleResultModel

/**
 * Lets the AI suggest a classification for one document. The matching rules come first: when they
 * decide category and correspondent, the suggestion is made without the AI; otherwise the AI gets
 * their values as fixed and they override its answer. The suggestion is stored as a task for the
 * user's review; nothing in the catalog or on disk changes, except the document's text in
 * `.dms-text/`, which search uses.
 */
class SuggestClassification(
    private val store: CatalogStore,
    private val reader: DocumentContentReader,
    private val classifier: DocumentClassifier,
    private val tasks: TaskStore,
    private val texts: TextStore,
    private val replicator: Replicator,
    private val clock: Clock,
) {

    suspend operator fun invoke(documentId: DocumentIdModel): SuggestOutcome {
        val library = store.current() ?: return SuggestOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return SuggestOutcome.NotFound
        val content = document.takeIf { it.kind.readableByAi }
            ?.let { reader.read(folder = library.folders.primary, document = it) }
        val rules = rulesFor(library = library, document = document, content = content)
        if (rules.isSufficient) {
            // The rules decided category and correspondent; the AI is not needed.
            return suggested(
                library = library,
                content = content,
                suggestion = RuleSuggestions.of(document = document, rules = rules, catalog = library.catalog),
            )
        }
        if (content == null) {
            return SuggestOutcome.NotReadable(detail = null)
        }
        if (content is DocumentContent.NotReadable) {
            return SuggestOutcome.NotReadable(detail = content.reason)
        }
        val request = request(catalog = library.catalog, document = document, content = content)
            .copy(fixedValues = rules.takeUnless { it.isEmpty })
        return when (val result = classifier.suggest(request = request)) {
            is ClassificationResult.Failed -> SuggestOutcome.Failed(reason = result.reason, detail = result.detail)
            is ClassificationResult.Suggested -> {
                val suggestion = normalized(
                    suggestion = result.suggestion,
                    request = request,
                    catalog = library.catalog,
                    document = document,
                )
                suggested(
                    library = library,
                    content = content,
                    suggestion = RuleSuggestions.over(
                        suggestion = suggestion,
                        rules = rules,
                        catalog = library.catalog,
                    ),
                )
            }
        }
    }

    private suspend fun suggested(
        library: LibrarySnapshotModel,
        content: DocumentContent?,
        suggestion: ClassificationSuggestionModel,
    ): SuggestOutcome {
        tasks.saveSuggestion(suggestion = suggestion)
        storeText(library = library, documentId = suggestion.documentId, content = content, suggestion = suggestion)
        return SuggestOutcome.Suggested(suggestion = suggestion)
    }

    /** The rules on the document's text as far as it is known: its text layer or transcription, name and title. */
    private suspend fun rulesFor(
        library: LibrarySnapshotModel,
        document: DocumentModel,
        content: DocumentContent?,
    ): RuleResultModel {
        val settings = library.catalog.settings
        if (settings.rules.isEmpty() && settings.correspondents.isEmpty()) {
            return RuleResultModel(correspondent = document.metadata.correspondent)
        }
        val text = (content as? DocumentContent.Text)?.text
            ?: texts.read(folder = library.folders.primary, documentId = document.id)
        val haystack = listOfNotNull(
            text,
            document.path.fileName,
            document.metadata.title,
        ).joinToString(separator = "\n")
        return RuleMatcher.apply(
            rules = settings.rules,
            correspondents = settings.correspondents,
            document = document,
            text = haystack,
        )
    }

    private fun request(
        catalog: CatalogModel,
        document: DocumentModel,
        content: DocumentContent,
    ): ClassificationRequestModel {
        val others = catalog.documents.values.filter { it.id != document.id }
        val inbox = catalog.settings.layout.inbox
        val outline = if (inbox.contains(candidate = document.path) && inbox.aiTargetFolder) {
            FolderOutlineModel.of(catalog = catalog, excluded = listOf(inbox.path))
        } else {
            null
        }
        return ClassificationRequestModel(
            documentId = document.id,
            currentFileName = document.path.fileName,
            content = content,
            categories = catalog.categories.values.sortedBy { it.name },
            tags = catalog.tags,
            folders = others.mapNotNull { it.path.parent }.filterNot { it.isRoot }.distinct().sortedBy { it.value },
            fileNameExamples = others.sortedByDescending { it.addedAt }
                .take(n = FILE_NAME_EXAMPLES)
                .map { it.path.fileName },
            today = clock.today(),
            outline = outline,
            fieldDefinitions = catalog.settings.fieldDefinitions,
            defaultCurrency = catalog.settings.layout.defaultCurrency,
        )
    }

    /**
     * Ties the suggestion to the document and checks it against the catalog as it is now. A target
     * folder is only kept for inbox documents and only where the outline allows it.
     */
    private fun normalized(
        suggestion: ClassificationSuggestionModel,
        request: ClassificationRequestModel,
        catalog: CatalogModel,
        document: DocumentModel,
    ): ClassificationSuggestionModel {
        val category = suggestion.category
        val knownCategory = category !is SuggestedCategory.Existing || category.categoryId in catalog.categories
        val target = suggestion.targetFolder?.let { folder ->
            val outline = request.outline ?: return@let null
            when {
                outline.contains(path = folder.path) -> folder.copy(isNew = false)
                outline.canCreate(path = folder.path) -> folder.copy(isNew = true)
                else -> null
            }
        }
        return suggestion.copy(
            documentId = document.id,
            category = if (knownCategory) category else null,
            tags = suggestion.tags.map { it.copy(isNew = it.name !in catalog.tags) },
            targetFolder = target,
            targetReason = suggestion.targetReason.takeIf { target != null },
            fields = suggestion.fields.filter { (key, value) -> catalog.settings.field(key = key)?.type == value.type },
        )
    }

    /** The text is derived data: a failure to store it does not fail the suggestion. */
    private suspend fun storeText(
        library: LibrarySnapshotModel,
        documentId: DocumentIdModel,
        content: DocumentContent?,
        suggestion: ClassificationSuggestionModel,
    ) {
        val text = (content as? DocumentContent.Text)?.text ?: suggestion.transcription
        if (text.isNullOrBlank()) {
            return
        }
        replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> texts.write(folder = primary, documentId = documentId, text = text) },
            followUp = FolderOperation.CopyText(documentId = documentId),
        )
    }

    private companion object {
        const val FILE_NAME_EXAMPLES = 20
    }
}
