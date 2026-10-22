package dms.domain.rule.match

import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.match.model.RuleResultModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleModel
import java.util.Locale

/**
 * Applies the matching rules to one document. Pure: the document, its text and the rules in, what
 * the rules propose out.
 *
 * The correspondent is the document's own, or the first known correspondent whose name or alias the
 * text contains. Where several rules decide the same value, the more specific rule wins, then a manual
 * rule over a learned one, then the earlier rule; tags of every matching rule are united.
 */
object RuleMatcher {

    fun apply(
        rules: List<MatchingRuleModel>,
        correspondents: List<CorrespondentModel>,
        document: DocumentModel,
        text: String,
    ): RuleResultModel {
        val correspondent = document.metadata.correspondent
            ?: correspondents.firstOrNull { it.appearsIn(text = text) }?.name
        val matching = rules
            .filter { rule ->
                rule.enabled && rule.conditions.all {
                    holds(condition = it, document = document, text = text, correspondent = correspondent)
                }
            }
            .sortedWith(
                compareByDescending<MatchingRuleModel> { it.specificity }.thenBy { it.origin != RuleOrigin.MANUAL },
            )
        if (matching.isEmpty()) {
            return RuleResultModel(correspondent = correspondent)
        }
        val category = matching.firstOrNull { it.actions.categoryId != null }
        val folder = matching.firstOrNull { it.actions.targetFolder != null }
        val tagging = matching.filter { it.actions.tags.isNotEmpty() }
        val fields = mutableMapOf<FieldKeyModel, FieldValue>()
        matching.reversed().forEach { fields.putAll(from = it.actions.fields) }
        val sources = buildMap {
            category?.let { put(key = RuleTarget.CATEGORY, value = it.id) }
            folder?.let { put(key = RuleTarget.TARGET_FOLDER, value = it.id) }
            tagging.firstOrNull()?.let { put(key = RuleTarget.TAGS, value = it.id) }
            matching.firstOrNull { it.actions.fields.isNotEmpty() }?.let { put(key = RuleTarget.FIELDS, value = it.id) }
        }
        return RuleResultModel(
            correspondent = correspondent,
            categoryId = category?.actions?.categoryId,
            tags = tagging.flatMap { it.actions.tags }.toSet(),
            fields = fields,
            targetFolder = folder?.actions?.targetFolder,
            sources = sources,
        )
    }

    private fun holds(
        condition: RuleCondition,
        document: DocumentModel,
        text: String,
        correspondent: String?,
    ): Boolean {
        return when (condition) {
            is RuleCondition.CorrespondentIs -> correspondent != null && sameName(
                first = correspondent,
                second = condition.name,
            )
            is RuleCondition.TextContains -> {
                val haystack = text.lowercase(Locale.ROOT)
                val found = condition.words.map { haystack.contains(other = it.trim().lowercase(Locale.ROOT)) }
                if (condition.requireAll) found.all { it } else found.any { it }
            }

            is RuleCondition.FileNameMatches -> globOf(pattern = condition.glob).matches(input = document.path.fileName)
        }
    }

    private fun sameName(first: String, second: String): Boolean {
        return first.trim().lowercase(Locale.ROOT) == second.trim().lowercase(Locale.ROOT)
    }

    /** `*` any characters, `?` one character, everything else literally, ignoring case. */
    private fun globOf(pattern: String): Regex {
        val regex = pattern.trim().map { character ->
            when (character) {
                '*' -> ".*"
                '?' -> "."
                else -> Regex.escape(literal = character.toString())
            }
        }.joinToString(separator = "")
        return Regex(pattern = regex, option = RegexOption.IGNORE_CASE)
    }
}
