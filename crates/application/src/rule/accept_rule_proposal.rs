use std::sync::Arc;

use dms_domain::rule::{
    Correspondent, MatchingRule, MatchingRuleId, RuleActions, RuleCondition, RuleOrigin, RuleProposal,
};

use crate::catalog::ChangeSettings;
use crate::common::OperationOutcome;
use crate::port::system::IdGenerator;

/// Turns a proposal the user accepted into a learned rule: documents of the correspondent get its category and tags
/// proposed. The correspondent joins the known correspondents, so it is also recognized in the text of documents
/// that do not name it yet.
pub struct AcceptRuleProposal {
    change_settings: Arc<ChangeSettings>,
    ids: Arc<dyn IdGenerator>,
}

impl AcceptRuleProposal {
    pub fn new(change_settings: Arc<ChangeSettings>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { change_settings, ids }
    }

    pub fn run(&self, proposal: &RuleProposal) -> OperationOutcome {
        let rule = MatchingRule::new(
            MatchingRuleId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
            proposal.correspondent.clone(),
            vec![RuleCondition::CorrespondentIs { name: proposal.correspondent.clone() }],
            RuleActions {
                category_id: proposal.category_id.clone(),
                tags: proposal.tags.clone(),
                ..RuleActions::default()
            },
            RuleOrigin::Learned,
        );
        let rule = match rule {
            Ok(rule) => rule,
            Err(error) => return OperationOutcome::InvalidInput(error),
        };
        let correspondent = match Correspondent::new(proposal.correspondent.clone(), Vec::new()) {
            Ok(correspondent) => correspondent,
            Err(error) => return OperationOutcome::InvalidInput(error),
        };
        self.change_settings.run(&format!("Rule {}", rule.name), |settings| {
            let known = settings.correspondents.iter().any(|it| it.key() == correspondent.key());
            let with_correspondent =
                if known { settings.clone() } else { settings.with_correspondent(correspondent.clone()) };
            with_correspondent.with_rule(rule.clone())
        })
    }
}
