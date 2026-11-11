//! Matching rules: what the library proposes for documents that look alike.

pub mod learn;
pub mod matching;
pub mod model;
mod rule_condition;
mod rule_origin;

pub use learn::{RuleLearner, RuleProposal};
pub use matching::{RuleMatcher, RuleResult, RuleTarget};
pub use model::{Correspondent, MatchingRule, MatchingRuleId, RuleActions};
pub use rule_condition::RuleCondition;
pub use rule_origin::RuleOrigin;
