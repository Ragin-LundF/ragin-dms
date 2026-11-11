use dms_domain::catalog::Catalog;
use dms_domain::classification::{ClassificationSuggestion, SuggestedCategory, SuggestedFolder, SuggestedTag};
use dms_domain::document::Document;
use dms_domain::library::RelativePath;
use dms_domain::rule::RuleResult;
use dms_domain::rule::matching::RuleTarget;
use indexmap::IndexSet;

/// What the matching rules add to a suggestion: a suggestion of their own when they decided category and
/// correspondent, or their values laid over the AI's, which they override. Tags are united.
///
/// A rule's target folder always wins over the AI's folder and covers its subfolders: a document already in the
/// target folder or below it stays where it is. A rule whose target is the top level therefore keeps every
/// document in its place.
pub struct RuleSuggestions;

impl RuleSuggestions {
    /// A suggestion made by the rules alone, without the AI.
    #[must_use]
    pub fn of(document: &Document, rules: &RuleResult, catalog: &Catalog) -> ClassificationSuggestion {
        let base = ClassificationSuggestion::empty(document.id.clone(), ClassificationSuggestion::RULES_MODEL);
        Self::over(base, rules, catalog)
    }

    /// `suggestion` with every value the rules decided in place of the AI's.
    #[must_use]
    pub fn over(
        suggestion: ClassificationSuggestion,
        rules: &RuleResult,
        catalog: &Catalog,
    ) -> ClassificationSuggestion {
        if rules.is_empty() {
            return suggestion;
        }
        let category_id = rules.category_id.clone().filter(|id| catalog.categories().contains_key(id));
        let rule_tags =
            rules.tags.iter().map(|tag| SuggestedTag { name: tag.clone(), is_new: !catalog.tags().contains(tag) });
        let mut seen = IndexSet::new();
        let tags =
            rule_tags.chain(suggestion.tags.iter().cloned()).filter(|tag| seen.insert(tag.name.clone())).collect();
        let mut fields = suggestion.fields.clone();
        fields.extend(rules.fields.iter().map(|(key, value)| (key.clone(), value.clone())));
        let target_folder = match &rules.target_folder {
            Some(target) => folder_of(target, catalog.documents().get(&suggestion.document_id), catalog),
            None => suggestion.target_folder.clone(),
        };
        let rule_notes = rules
            .sources
            .iter()
            .filter(|(target, _)| **target != RuleTarget::Category || category_id.is_some())
            .map(|(target, id)| {
                let name = catalog.settings().rules.iter().find(|rule| &rule.id == id).map(|rule| rule.name.clone());
                (*target, name.unwrap_or_else(|| id.value().to_owned()))
            })
            .collect();
        ClassificationSuggestion {
            category: category_id.map(SuggestedCategory::Existing).or(suggestion.category.clone()),
            tags,
            correspondent: rules.correspondent.clone().or(suggestion.correspondent.clone()),
            fields,
            target_folder,
            target_reason: if rules.target_folder.is_some() { None } else { suggestion.target_reason.clone() },
            rule_notes,
            ..suggestion
        }
    }
}

/// The folder to file `document` in for a rule's `target`; `None` when it already lies in or below it.
fn folder_of(target: &RelativePath, document: Option<&Document>, catalog: &Catalog) -> Option<SuggestedFolder> {
    if document.is_some_and(|document| document.path.is_inside(target)) {
        return None;
    }
    let exists = catalog.documents().values().any(|document| document.path.is_inside(target));
    Some(SuggestedFolder { path: target.clone(), is_new: !exists })
}
