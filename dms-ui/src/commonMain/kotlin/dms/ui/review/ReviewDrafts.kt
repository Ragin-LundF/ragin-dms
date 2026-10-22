package dms.ui.review

import dms.application.classification.model.ApprovedCategory
import dms.application.classification.model.SuggestionApprovalModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.review.model.ReviewDraftModel
import dms.ui.review.model.ReviewFieldModel
import dms.ui.session.OutcomeMessages
import java.time.LocalDate
import java.time.format.DateTimeParseException

/** Between a suggestion and what the user approves: the draft to edit, and the approval it makes. */
object ReviewDrafts {

    private const val SEPARATOR = " / "

    /**
     * The suggestion as a draft; a field the AI left empty starts unchecked, and the file name only
     * starts checked when [acceptFileName] is set.
     */
    fun draftOf(
        suggestion: ClassificationSuggestionModel,
        catalog: CatalogModel,
        acceptFileName: Boolean,
    ): ReviewDraftModel {
        val tags = suggestion.tags.joinToString(separator = ", ") { it.name.value }
        return ReviewDraftModel(
            title = field(value = suggestion.title),
            documentDate = field(value = suggestion.documentDate?.toString()),
            correspondent = field(value = suggestion.correspondent),
            category = field(value = categoryText(category = suggestion.category, catalog = catalog)),
            tags = field(value = tags.ifEmpty { null }),
            fileName = field(value = suggestion.fileName).let { it.copy(accepted = it.accepted && acceptFileName) },
            targetFolder = field(value = suggestion.targetFolder?.path?.value),
            fields = suggestion.fields.mapValues { (_, value) -> field(value = FieldValues.format(value = value)) },
        )
    }

    /** The document as it is, for comparison. */
    fun currentOf(document: DocumentModel, catalog: CatalogModel): ReviewDraftModel {
        val category = document.categoryId?.let { catalog.categories[it] }
        return ReviewDraftModel(
            title = ReviewFieldModel(accepted = false, value = document.metadata.title.orEmpty()),
            documentDate = ReviewFieldModel(
                accepted = false,
                value = document.metadata.documentDate?.toString().orEmpty(),
            ),
            correspondent = ReviewFieldModel(accepted = false, value = document.metadata.correspondent.orEmpty()),
            category = ReviewFieldModel(
                accepted = false,
                value = category?.let { pathOf(category = it, catalog = catalog) }.orEmpty(),
            ),
            tags = ReviewFieldModel(accepted = false, value = document.tags.sorted().joinToString(separator = ", ")),
            fileName = ReviewFieldModel(accepted = false, value = document.path.fileName),
            targetFolder = ReviewFieldModel(accepted = false, value = document.path.parent?.value.orEmpty()),
            fields = document.fields.mapValues { (_, value) ->
                ReviewFieldModel(accepted = false, value = FieldValues.format(value = value))
            },
        )
    }

    /** The approval for the checked fields, or why it cannot be made. */
    fun approvalOf(
        draft: ReviewDraftModel,
        current: ReviewDraftModel,
        catalog: CatalogModel,
    ): Result<SuggestionApprovalModel> {
        return runCatching {
            SuggestionApprovalModel(
                title = draft.title.taken(),
                documentDate = draft.documentDate.taken()?.let { dateOf(text = it) },
                correspondent = draft.correspondent.taken(),
                category = draft.category.taken()?.let { categoryOf(text = it, catalog = catalog) },
                tags = draft.tags.taken()?.let { tagsOf(text = it) }.orEmpty(),
                fileName = draft.fileName.taken()
                    ?.let { fileNameOf(name = it) }
                    ?.takeIf { it != current.fileName.value },
                targetFolder = draft.targetFolder.taken()
                    ?.let { folderOf(text = it) }
                    ?.takeIf { it.value != current.targetFolder.value },
                fields = fieldsOf(drafts = draft.fields, catalog = catalog),
            )
        }
    }

    /** Every folder the catalog's documents are in, sorted, to pick a target folder from. */
    fun folderPathsOf(catalog: CatalogModel): List<String> {
        return catalog.documents.values
            .flatMap { document -> generateSequence(seed = document.path.parent) { it.parent }.toList() }
            .filterNot { it.isRoot }
            .map { it.value }
            .distinct()
            .sortedWith(comparator = String.CASE_INSENSITIVE_ORDER)
    }

    /** Every category as a path like `Parent / Child`, sorted, to pick from. */
    fun categoryPathsOf(catalog: CatalogModel): List<String> {
        return catalog.categories.values
            .map { pathOf(category = it, catalog = catalog) }
            .sortedWith(comparator = String.CASE_INSENSITIVE_ORDER)
    }

    /** Whether approving [text] creates a category, because no path or name of [paths] matches it. */
    fun isNewCategory(text: String, paths: List<String>): Boolean {
        val typed = text.trim()
        return typed.isNotEmpty() && paths.none { path ->
            path.equals(typed, ignoreCase = true) ||
                path.substringAfterLast(delimiter = "/").trim().equals(typed, ignoreCase = true)
        }
    }

    fun pathOf(category: CategoryModel, catalog: CatalogModel): String {
        val names = generateSequence(seed = category) { current -> current.parentId?.let { catalog.categories[it] } }
            .take(n = catalog.categories.size)
            .map { it.name }
            .toList()
        return names.reversed().joinToString(separator = SEPARATOR)
    }

    private fun categoryText(category: SuggestedCategory?, catalog: CatalogModel): String? {
        return when (category) {
            null -> null
            is SuggestedCategory.Existing -> catalog.categories[category.categoryId]?.let {
                pathOf(category = it, catalog = catalog)
            }

            is SuggestedCategory.New -> {
                val parent = category.parentId?.let { catalog.categories[it] }
                listOfNotNull(parent?.let { pathOf(category = it, catalog = catalog) }, category.name)
                    .joinToString(separator = SEPARATOR)
            }
        }
    }

    /**
     * An existing category by path or name, ignoring case; otherwise the typed path, whose missing
     * levels approving creates.
     */
    private fun categoryOf(text: String, catalog: CatalogModel): ApprovedCategory {
        val existing = catalog.categories.values.firstOrNull { category ->
            pathOf(category = category, catalog = catalog).equals(text, ignoreCase = true) ||
                category.name.equals(text, ignoreCase = true)
        }
        if (existing != null) {
            return ApprovedCategory.Existing(categoryId = existing.id)
        }
        val names = text.split('/').map { it.trim() }
        require(value = names.none { it.isEmpty() }) { "Every level of the category \"$text\" needs a name" }
        return ApprovedCategory.Path(names = names)
    }

    /** The checked field values, each read with its field's type; one that does not fit names the field. */
    private fun fieldsOf(
        drafts: Map<FieldKeyModel,
        ReviewFieldModel>,
        catalog: CatalogModel,
    ): Map<FieldKeyModel, FieldValue> {
        val currency = catalog.settings.layout.defaultCurrency
        return drafts.mapNotNull { (key, draft) ->
            val text = draft.taken() ?: return@mapNotNull null
            val definition = catalog.settings.field(key = key) ?: return@mapNotNull null
            val value = requireNotNull(
                value = FieldValues.parse(type = definition.type, raw = text, defaultCurrency = currency),
            ) {
                "\"${definition.label}\" does not fit the field's type"
            }
            key to value
        }.toMap()
    }

    private fun folderOf(text: String): RelativePathModel {
        val segments = text.split('/').map { it.trim() }.filter { it.isNotEmpty() }
        segments.forEach { name ->
            val violation = FileNamePolicy.check(name = name)
            require(value = violation == null) {
                "Folder \"$name\": ${violation?.let { OutcomeMessages.nameProblem(violation = it) }}"
            }
        }
        return RelativePathModel.fromSegments(segments = segments)
    }

    private fun dateOf(text: String): LocalDate {
        return try {
            LocalDate.parse(text)
        } catch (exception: DateTimeParseException) {
            throw IllegalArgumentException("Write the date as YYYY-MM-DD", exception)
        }
    }

    private fun tagsOf(text: String): Set<TagNameModel> {
        return text.split(',').filter { it.isNotBlank() }.map { raw ->
            runCatching { TagNameModel.of(raw = raw) }.getOrElse { failure ->
                throw IllegalArgumentException("Tag \"${raw.trim()}\": ${failure.message}", failure)
            }
        }.toSet()
    }

    private fun fileNameOf(name: String): String {
        val violation = FileNamePolicy.check(name = name)
        require(value = violation == null) { violation?.let { OutcomeMessages.nameProblem(violation = it) }.orEmpty() }
        return name
    }

    private fun field(value: String?): ReviewFieldModel {
        return ReviewFieldModel(accepted = !value.isNullOrBlank(), value = value.orEmpty())
    }

    private fun ReviewFieldModel.taken(): String? {
        return value.trim().takeIf { accepted && it.isNotEmpty() }
    }
}
