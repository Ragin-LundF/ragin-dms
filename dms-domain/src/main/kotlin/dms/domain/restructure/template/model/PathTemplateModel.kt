package dms.domain.restructure.template.model

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.restructure.template.PlaceholderName
import dms.domain.restructure.template.TemplateParse
import dms.domain.restructure.template.TemplatePart
import dms.domain.restructure.template.TemplateProblem
import dms.domain.restructure.template.TemplateRendering
import dms.domain.taxonomy.model.CategoryModel
import java.util.Locale

/**
 * Where a document belongs, written like `{category}/{year|Undated}/`: folder names separated by
 * `/`, with placeholders in braces and an optional fallback after `|`. Every rendered folder name is
 * made valid with [FileNamePolicy.sanitize], so a value never breaks the path.
 */
class PathTemplateModel private constructor(
    val text: String,
    private val segments: List<List<TemplatePart>>,
) {

    /** The placeholders the template uses, for previews and skip reasons. */
    val placeholders: List<TemplatePart.Placeholder>
        get() {
            return segments.flatten().filterIsInstance<TemplatePart.Placeholder>()
        }

    /** The folder [document] belongs in, or the placeholder it has no value for. */
    fun render(document: DocumentModel, catalog: CatalogModel): TemplateRendering {
        val folders = mutableListOf<String>()
        segments.forEach { parts ->
            val pieces = parts.map { part ->
                when (part) {
                    is TemplatePart.Literal -> listOf(part.text)
                    is TemplatePart.Placeholder -> valueOf(placeholder = part, document = document, catalog = catalog)
                        ?: return TemplateRendering.Missing(placeholder = part.name)
                }
            }
            folders += join(pieces = pieces)
        }
        val names = folders.filter { it.isNotBlank() }.map { FileNamePolicy.sanitize(name = it) }
        return TemplateRendering.Rendered(folder = RelativePathModel.fromSegments(segments = names))
    }

    override fun equals(other: Any?): Boolean {
        return other is PathTemplateModel && other.text == text
    }

    override fun hashCode(): Int {
        return text.hashCode()
    }

    override fun toString(): String {
        return text
    }

    /**
     * Joins the pieces of one template segment. A piece with several folder names (a category path)
     * contributes its first names as folders of their own and its last to the segment's text.
     */
    private fun join(pieces: List<List<String>>): List<String> {
        val result = mutableListOf("")
        pieces.forEach { piece ->
            result[result.lastIndex] = result.last() + piece.first()
            piece.drop(n = 1).forEach { result += it }
        }
        return result
    }

    /** The value of [placeholder] as one or more folder names, or `null` if there is none. */
    private fun valueOf(
        placeholder: TemplatePart.Placeholder,
        document: DocumentModel,
        catalog: CatalogModel,
    ): List<String>? {
        val date = document.metadata.documentDate
        val value = when (placeholder.name) {
            PlaceholderName.CATEGORY -> categoryPath(document = document, catalog = catalog)
            PlaceholderName.CATEGORY_TOP -> categoryPath(document = document, catalog = catalog)?.take(n = 1)
            PlaceholderName.YEAR -> date?.let { listOf(it.year.toString()) }
            PlaceholderName.MONTH -> date?.let { listOf("%02d".format(Locale.ROOT, it.monthValue)) }
            PlaceholderName.CORRESPONDENT -> document.metadata.correspondent?.let { listOf(it) }
            PlaceholderName.KIND -> listOf(kindName(document = document))
            PlaceholderName.FIELD -> document.fields[placeholder.fieldKey]?.let {
                listOf(FieldValues.format(value = it))
            }
        }
        return value?.map { FileNamePolicy.sanitize(name = it) } ?: placeholder.fallback?.let { listOf(it) }
    }

    private fun kindName(document: DocumentModel): String {
        return document.kind.name.lowercase(Locale.ROOT).replaceFirstChar { it.titlecase(Locale.ROOT) }
    }

    private fun categoryPath(document: DocumentModel, catalog: CatalogModel): List<String>? {
        val names = mutableListOf<String>()
        val visited = mutableSetOf<CategoryModel>()
        var current = document.categoryId?.let { catalog.categories[it] }
        while (current != null && visited.add(element = current)) {
            names.add(index = 0, element = current.name)
            current = current.parentId?.let { catalog.categories[it] }
        }
        return names.takeIf { it.isNotEmpty() }
    }

    companion object {

        private const val FIELD_PREFIX = "field:"

        /** Reads [raw]; surrounding blanks and a leading or trailing `/` are ignored. */
        fun parse(raw: String): TemplateParse {
            val text = raw.trim().trim('/')
            if (text.isEmpty()) {
                return TemplateParse.Invalid(problem = TemplateProblem.EMPTY)
            }
            val segments = mutableListOf<List<TemplatePart>>()
            text.split('/').forEach { segment ->
                when (val parsed = parseSegment(segment = segment.trim())) {
                    is SegmentParse.Parts -> segments += parsed.parts
                    is SegmentParse.Problem -> return TemplateParse.Invalid(problem = parsed.problem)
                }
            }
            return TemplateParse.Parsed(template = PathTemplateModel(text = text, segments = segments))
        }

        private fun parseSegment(segment: String): SegmentParse {
            val parts = mutableListOf<TemplatePart>()
            var rest = segment
            while (rest.isNotEmpty()) {
                val open = rest.indexOf(char = '{')
                if (open < 0) {
                    parts += TemplatePart.Literal(text = rest)
                    break
                }
                if (open > 0) {
                    parts += TemplatePart.Literal(text = rest.substring(startIndex = 0, endIndex = open))
                }
                val close = rest.indexOf(char = '}', startIndex = open)
                if (close < 0) {
                    return SegmentParse.Problem(problem = TemplateProblem.UNCLOSED_BRACE)
                }
                when (val placeholder = placeholderOf(body = rest.substring(startIndex = open + 1, endIndex = close))) {
                    is PlaceholderParse.Found -> parts += placeholder.placeholder
                    is PlaceholderParse.Problem -> return SegmentParse.Problem(problem = placeholder.problem)
                }
                rest = rest.substring(startIndex = close + 1)
            }
            val literal = parts.filterIsInstance<TemplatePart.Literal>().joinToString(separator = "") { it.text }
            if (parts.size == 1 && literal.isNotEmpty() && FileNamePolicy.check(name = literal) != null) {
                return SegmentParse.Problem(problem = TemplateProblem.INVALID_FOLDER_NAME)
            }
            if (literal.any { it in "<>:\"\\|?*" }) {
                return SegmentParse.Problem(problem = TemplateProblem.INVALID_FOLDER_NAME)
            }
            return SegmentParse.Parts(parts = parts)
        }

        private fun placeholderOf(body: String): PlaceholderParse {
            val token = body.substringBefore(delimiter = '|').trim()
            val fallback = body.substringAfter(
                delimiter = '|',
                missingDelimiterValue = "",
            ).trim().takeIf { it.isNotEmpty() }
            if (token.startsWith(prefix = FIELD_PREFIX)) {
                val key = token.removePrefix(prefix = FIELD_PREFIX)
                if (!FIELD_KEY.matches(input = key)) {
                    return PlaceholderParse.Problem(problem = TemplateProblem.INVALID_FIELD_KEY)
                }
                return PlaceholderParse.Found(
                    placeholder = TemplatePart.Placeholder(
                        name = PlaceholderName.FIELD,
                        fieldKey = FieldKeyModel(value = key),
                        fallback = fallback,
                    ),
                )
            }
            val name = PlaceholderName.of(token = token)
            if (name == null || name == PlaceholderName.FIELD) {
                return PlaceholderParse.Problem(problem = TemplateProblem.UNKNOWN_PLACEHOLDER)
            }
            return PlaceholderParse.Found(placeholder = TemplatePart.Placeholder(name = name, fallback = fallback))
        }

        private val FIELD_KEY = Regex(pattern = "^[a-z][a-z0-9_-]*$")
    }

    private sealed interface SegmentParse {
        data class Parts(val parts: List<TemplatePart>) : SegmentParse
        data class Problem(val problem: TemplateProblem) : SegmentParse
    }

    private sealed interface PlaceholderParse {
        data class Found(val placeholder: TemplatePart.Placeholder) : PlaceholderParse
        data class Problem(val problem: TemplateProblem) : PlaceholderParse
    }
}
