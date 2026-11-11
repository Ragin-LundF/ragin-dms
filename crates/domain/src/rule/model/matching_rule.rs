use crate::DomainError;
use crate::error::require;
use crate::field::FieldValue;
use crate::rule::{MatchingRuleId, RuleActions, RuleCondition, RuleOrigin};
use crate::text;

/// "When a document looks like this, propose that": every [`Self::conditions`] entry must hold, and the
/// [`Self::actions`] become part of the suggestion the user reviews. A disabled rule is kept but not used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchingRule {
    pub id: MatchingRuleId,
    pub name: String,
    pub conditions: Vec<RuleCondition>,
    pub actions: RuleActions,
    pub origin: RuleOrigin,
    pub enabled: bool,
}

impl MatchingRule {
    /// An enabled rule.
    ///
    /// # Errors
    /// Fails like [`Self::validate`].
    pub fn new(
        id: MatchingRuleId,
        name: impl Into<String>,
        conditions: Vec<RuleCondition>,
        actions: RuleActions,
        origin: RuleOrigin,
    ) -> Result<Self, DomainError> {
        let rule = Self { id, name: name.into(), conditions, actions, origin, enabled: true };
        rule.validate()?;
        Ok(rule)
    }

    /// # Errors
    /// Fails for a blank name, no conditions, no actions, or an invalid condition or field value.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.name), || "Rule name must not be blank".to_owned())?;
        require(!self.conditions.is_empty(), || "A rule needs at least one condition".to_owned())?;
        require(!self.actions.is_empty(), || "A rule needs at least one action".to_owned())?;
        self.conditions.iter().try_for_each(RuleCondition::validate)?;
        self.actions.fields.values().try_for_each(FieldValue::validate)
    }

    /// More conditions make a rule more specific; a more specific rule wins a conflict.
    #[must_use]
    pub fn specificity(&self) -> usize {
        self.conditions
            .iter()
            .map(|condition| match condition {
                RuleCondition::TextContains { words, require_all: true } => words.len(),
                _ => 1,
            })
            .sum()
    }
}
