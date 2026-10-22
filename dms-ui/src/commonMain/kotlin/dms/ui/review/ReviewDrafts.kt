package dms.ui.review

import dms.application.classification.model.ApprovedCategory
import dms.application.classification.model.SuggestionApproval
import dms.domain.catalog.Catalog
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.document.Document
import dms.domain.library.FileNamePolicy
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.TagName
import dms.ui.review.model.ReviewDraft
import dms.ui.review.model.ReviewField
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
    fun draftOf(suggestion: ClassificationSuggestion, catalog: Catalog, acceptFileName: Boolean): ReviewDraft {
        val tags = suggestion.tags.joinToString(separator = ", ") { it.name.value }
        return ReviewDraft(
            title = field(value = suggestion.title),
            documentDate = field(value = suggestion.documentDate?.toString()),
            correspondent = field(value = suggestion.correspondent),
            category = field(value = categoryText(category = suggestion.category, catalog = catalog)),
            tags = field(value = tags.ifEmpty { null }),
            fileName = field(value = suggestion.fileName).let { it.copy(accepted = it.accepted && acceptFileName) },
        )
    }

    /** The document as it is, for comparison. */
    fun currentOf(document: Document, catalog: Catalog): ReviewDraft {
        val category = document.categoryId?.let { catalog.categories[it] }
        return ReviewDraft(
            title = ReviewField(accepted = false, value = document.metadata.title.orEmpty()),
            documentDate = ReviewField(accepted = false, value = document.metadata.documentDate?.toString().orEmpty()),
            correspondent = ReviewField(accepted = false, value = document.metadata.correspondent.orEmpty()),
            category = ReviewField(
                accepted = false,
                value = category?.let { pathOf(category = it, catalog = catalog) }.orEmpty(),
            ),
            tags = ReviewField(accepted = false, value = document.tags.sorted().joinToString(separator = ", ")),
            fileName = ReviewField(accepted = false, value = document.path.fileName),
        )
    }

    /** The approval for the checked fields, or why it cannot be made. */
    fun approvalOf(draft: ReviewDraft, current: ReviewDraft, catalog: Catalog): Result<SuggestionApproval> {
        return runCatching {
            SuggestionApproval(
                title = draft.title.taken(),
                documentDate = draft.documentDate.taken()?.let { dateOf(text = it) },
                correspondent = draft.correspondent.taken(),
                category = draft.category.taken()?.let { categoryOf(text = it, catalog = catalog) },
                tags = draft.tags.taken()?.let { tagsOf(text = it) }.orEmpty(),
                fileName = draft.fileName.taken()
                    ?.let { fileNameOf(name = it) }
                    ?.takeIf { it != current.fileName.value },
            )
        }
    }

    /** Every category as a path like `Parent / Child`, sorted, to pick from. */
    fun categoryPathsOf(catalog: Catalog): List<String> {
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

    fun pathOf(category: Category, catalog: Catalog): String {
        val names = generateSequence(seed = category) { current -> current.parentId?.let { catalog.categories[it] } }
            .take(n = catalog.categories.size)
            .map { it.name }
            .toList()
        return names.reversed().joinToString(separator = SEPARATOR)
    }

    private fun categoryText(category: SuggestedCategory?, catalog: Catalog): String? {
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
    private fun categoryOf(text: String, catalog: Catalog): ApprovedCategory {
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

    private fun dateOf(text: String): LocalDate {
        return try {
            LocalDate.parse(text)
        } catch (exception: DateTimeParseException) {
            throw IllegalArgumentException("Write the date as YYYY-MM-DD", exception)
        }
    }

    private fun tagsOf(text: String): Set<TagName> {
        return text.split(',').filter { it.isNotBlank() }.map { raw ->
            runCatching { TagName.of(raw = raw) }.getOrElse { failure ->
                throw IllegalArgumentException("Tag \"${raw.trim()}\": ${failure.message}", failure)
            }
        }.toSet()
    }

    private fun fileNameOf(name: String): String {
        val violation = FileNamePolicy.check(name = name)
        require(value = violation == null) { violation?.let { OutcomeMessages.nameProblem(violation = it) }.orEmpty() }
        return name
    }

    private fun field(value: String?): ReviewField {
        return ReviewField(accepted = !value.isNullOrBlank(), value = value.orEmpty())
    }

    private fun ReviewField.taken(): String? {
        return value.trim().takeIf { accepted && it.isNotEmpty() }
    }
}
