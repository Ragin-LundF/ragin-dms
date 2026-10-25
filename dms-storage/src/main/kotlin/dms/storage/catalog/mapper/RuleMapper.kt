package dms.storage.catalog.mapper

import dms.domain.library.model.RelativePathModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.storage.catalog.dto.rule.RuleActionsDto
import dms.storage.catalog.dto.rule.RuleConditionDto
import dms.storage.catalog.dto.rule.RuleDto

/** Maps matching rules. */
object RuleMapper {

    private const val CORRESPONDENT = "correspondent"
    private const val TEXT_ANY = "textAny"
    private const val TEXT_ALL = "textAll"
    private const val FILE_NAME = "fileName"

    fun toDto(rule: MatchingRuleModel): RuleDto {
        return RuleDto(
            id = rule.id.value,
            name = rule.name,
            origin = rule.origin.name,
            enabled = rule.enabled,
            conditions = rule.conditions.map { toDto(condition = it) },
            actions = RuleActionsDto(
                categoryId = rule.actions.categoryId?.value,
                tags = rule.actions.tags.sorted().map { it.value },
                fields = FieldMapper.toDto(fields = rule.actions.fields),
                targetFolder = rule.actions.targetFolder?.value,
            ),
        )
    }

    fun toDomain(rule: RuleDto): MatchingRuleModel {
        return MatchingRuleModel(
            id = MatchingRuleIdModel(value = rule.id),
            name = rule.name,
            conditions = rule.conditions.map { toDomain(condition = it) },
            actions = RuleActionsModel(
                categoryId = rule.actions.categoryId?.let { CategoryIdModel(value = it) },
                tags = rule.actions.tags.map { TagNameModel.of(raw = it) }.toSet(),
                fields = FieldMapper.toDomain(fields = rule.actions.fields),
                targetFolder = rule.actions.targetFolder?.let { RelativePathModel.of(raw = it) },
            ),
            origin = RuleOrigin.valueOf(rule.origin),
            enabled = rule.enabled,
        )
    }

    private fun toDto(condition: RuleCondition): RuleConditionDto {
        return when (condition) {
            is RuleCondition.CorrespondentIs -> RuleConditionDto(type = CORRESPONDENT, name = condition.name)
            is RuleCondition.TextContains -> RuleConditionDto(
                type = if (condition.requireAll) TEXT_ALL else TEXT_ANY,
                words = condition.words,
            )

            is RuleCondition.FileNameMatches -> RuleConditionDto(type = FILE_NAME, glob = condition.glob)
        }
    }

    private fun toDomain(condition: RuleConditionDto): RuleCondition {
        return when (condition.type) {
            CORRESPONDENT -> RuleCondition.CorrespondentIs(name = requireNotNull(value = condition.name))
            TEXT_ANY -> RuleCondition.TextContains(words = condition.words, requireAll = false)
            TEXT_ALL -> RuleCondition.TextContains(words = condition.words, requireAll = true)
            FILE_NAME -> RuleCondition.FileNameMatches(glob = requireNotNull(value = condition.glob))
            else -> throw IllegalArgumentException("Unknown rule condition: ${condition.type}")
        }
    }
}
