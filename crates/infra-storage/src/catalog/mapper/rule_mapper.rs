//! Matching rules.

use dms_domain::library::RelativePath;
use dms_domain::rule::{MatchingRule, MatchingRuleId, RuleActions, RuleCondition};
use dms_domain::taxonomy::CategoryId;
use indexmap::IndexMap;

use crate::catalog::dto::rule::{RuleActionsDto, RuleConditionDto, RuleDto};
use crate::catalog::mapper::{field_mapper, query_mapper};
use crate::enum_names::{self, RULE_ORIGINS};
use crate::yaml::YamlError;

const CORRESPONDENT: &str = "correspondent";
const TEXT_ANY: &str = "textAny";
const TEXT_ALL: &str = "textAll";
const FILE_NAME: &str = "fileName";

pub fn rule_to_dto(rule: &MatchingRule) -> RuleDto {
    RuleDto {
        id: rule.id.value().to_owned(),
        name: rule.name.clone(),
        origin: enum_names::name_of(RULE_ORIGINS, rule.origin).to_owned(),
        enabled: rule.enabled,
        conditions: rule.conditions.iter().map(condition_to_dto).collect(),
        actions: RuleActionsDto {
            category_id: rule.actions.category_id.as_ref().map(|id| id.value().to_owned()),
            tags: query_mapper::sorted_tags(&rule.actions.tags),
            fields: field_mapper::values_to_dto(&rule.actions.fields),
            target_folder: rule.actions.target_folder.as_ref().map(|folder| folder.value().to_owned()),
            unknown: IndexMap::new(),
        },
        unknown: IndexMap::new(),
    }
}

pub fn rule_to_domain(rule: &RuleDto) -> Result<MatchingRule, YamlError> {
    let actions = &rule.actions;
    Ok(MatchingRule {
        id: MatchingRuleId::new(rule.id.as_str())?,
        name: rule.name.clone(),
        conditions: rule.conditions.iter().map(condition_to_domain).collect::<Result<_, _>>()?,
        actions: RuleActions {
            category_id: actions.category_id.as_deref().map(CategoryId::new).transpose()?,
            tags: query_mapper::tags_of(&actions.tags)?,
            fields: field_mapper::values_to_domain(&actions.fields)?,
            target_folder: actions.target_folder.as_deref().map(RelativePath::of).transpose()?,
        },
        origin: enum_names::parse(RULE_ORIGINS, &rule.origin)?,
        enabled: rule.enabled,
    })
}

fn condition_to_dto(condition: &RuleCondition) -> RuleConditionDto {
    let dto = |condition_type: &str| RuleConditionDto {
        condition_type: condition_type.to_owned(),
        name: None,
        words: Vec::new(),
        glob: None,
    };
    match condition {
        RuleCondition::CorrespondentIs { name } => RuleConditionDto { name: Some(name.clone()), ..dto(CORRESPONDENT) },
        RuleCondition::TextContains { words, require_all } => {
            RuleConditionDto { words: words.clone(), ..dto(if *require_all { TEXT_ALL } else { TEXT_ANY }) }
        }
        RuleCondition::FileNameMatches { glob } => RuleConditionDto { glob: Some(glob.clone()), ..dto(FILE_NAME) },
    }
}

fn condition_to_domain(condition: &RuleConditionDto) -> Result<RuleCondition, YamlError> {
    let required =
        |value: &Option<String>| value.clone().ok_or_else(|| YamlError("Required value was null.".to_owned()));
    match condition.condition_type.as_str() {
        CORRESPONDENT => Ok(RuleCondition::CorrespondentIs { name: required(&condition.name)? }),
        TEXT_ANY => Ok(RuleCondition::TextContains { words: condition.words.clone(), require_all: false }),
        TEXT_ALL => Ok(RuleCondition::TextContains { words: condition.words.clone(), require_all: true }),
        FILE_NAME => Ok(RuleCondition::FileNameMatches { glob: required(&condition.glob)? }),
        other => Err(YamlError(format!("Unknown rule condition: {other}"))),
    }
}
