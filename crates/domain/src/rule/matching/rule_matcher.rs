use indexmap::{IndexMap, IndexSet};

use crate::document::Document;
use crate::rule::{Correspondent, MatchingRule, RuleCondition, RuleOrigin, RuleResult, RuleTarget};
use crate::text;

/// Applies the matching rules to one document. Pure: the document, its text and the rules in, what the rules
/// propose out.
///
/// The correspondent is the document's own, or the first known correspondent whose name or alias the text
/// contains. Where several rules decide the same value, the more specific rule wins, then a manual rule over a
/// learned one, then the earlier rule; tags of every matching rule are united.
pub struct RuleMatcher;

impl RuleMatcher {
    #[must_use]
    pub fn apply(
        rules: &[MatchingRule],
        correspondents: &[Correspondent],
        document: &Document,
        text: &str,
    ) -> RuleResult {
        let correspondent = document.metadata.correspondent.clone().or_else(|| {
            correspondents.iter().find(|candidate| candidate.appears_in(text)).map(|found| found.name.clone())
        });
        let mut matching: Vec<&MatchingRule> = rules
            .iter()
            .filter(|rule| {
                rule.enabled
                    && rule
                        .conditions
                        .iter()
                        .all(|condition| holds(condition, document, text, correspondent.as_deref()))
            })
            .collect();
        matching.sort_by(|first, second| {
            second
                .specificity()
                .cmp(&first.specificity())
                .then((first.origin != RuleOrigin::Manual).cmp(&(second.origin != RuleOrigin::Manual)))
        });
        if matching.is_empty() {
            return RuleResult { correspondent, ..RuleResult::default() };
        }
        let category = matching.iter().find(|rule| rule.actions.category_id.is_some());
        let folder = matching.iter().find(|rule| rule.actions.target_folder.is_some());
        let tagging: Vec<&&MatchingRule> = matching.iter().filter(|rule| !rule.actions.tags.is_empty()).collect();
        let mut fields = IndexMap::new();
        for rule in matching.iter().rev() {
            fields.extend(rule.actions.fields.iter().map(|(key, value)| (key.clone(), value.clone())));
        }
        let mut sources = IndexMap::new();
        if let Some(rule) = category {
            sources.insert(RuleTarget::Category, rule.id.clone());
        }
        if let Some(rule) = folder {
            sources.insert(RuleTarget::TargetFolder, rule.id.clone());
        }
        if let Some(rule) = tagging.first() {
            sources.insert(RuleTarget::Tags, rule.id.clone());
        }
        if let Some(rule) = matching.iter().find(|rule| !rule.actions.fields.is_empty()) {
            sources.insert(RuleTarget::Fields, rule.id.clone());
        }
        RuleResult {
            correspondent,
            category_id: category.and_then(|rule| rule.actions.category_id.clone()),
            tags: tagging.iter().flat_map(|rule| rule.actions.tags.iter().cloned()).collect::<IndexSet<_>>(),
            fields,
            target_folder: folder.and_then(|rule| rule.actions.target_folder.clone()),
            sources,
        }
    }
}

fn holds(condition: &RuleCondition, document: &Document, text: &str, correspondent: Option<&str>) -> bool {
    match condition {
        RuleCondition::CorrespondentIs { name } => {
            correspondent.is_some_and(|correspondent| same_name(correspondent, name))
        }
        RuleCondition::TextContains { words, require_all } => {
            let haystack = text::lowercase(text);
            let mut found = words.iter().map(|word| haystack.contains(&text::lowercase(text::trim(word))));
            if *require_all { found.all(|it| it) } else { found.any(|it| it) }
        }
        RuleCondition::FileNameMatches { glob } => glob_matches(glob, document.path.file_name()),
    }
}

fn same_name(first: &str, second: &str) -> bool {
    text::lowercase(text::trim(first)) == text::lowercase(text::trim(second))
}

/// The Kotlin version builds a regex with `IGNORE_CASE`: `*` is `.*`, `?` is `.`, everything else literal. Java
/// ignores only ASCII case there, and `.` matches no line terminator; this matcher does the same.
fn glob_matches(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = text::trim(pattern).chars().collect();
    let name: Vec<char> = name.chars().collect();
    matches_from(&pattern, &name)
}

/// `matches[i][j]`: the pattern from `i` matches the name from `j`. Dynamic programming keeps many `*` cheap.
fn matches_from(pattern: &[char], name: &[char]) -> bool {
    let mut matches = vec![vec![false; name.len() + 1]; pattern.len() + 1];
    matches[pattern.len()][name.len()] = true;
    for i in (0..pattern.len()).rev() {
        for j in (0..=name.len()).rev() {
            let next = name.get(j);
            matches[i][j] = match pattern[i] {
                '*' => matches[i + 1][j] || next.is_some_and(|it| !is_line_terminator(*it) && matches[i][j + 1]),
                '?' => next.is_some_and(|it| !is_line_terminator(*it) && matches[i + 1][j + 1]),
                literal => next.is_some_and(|it| literal.eq_ignore_ascii_case(it) && matches[i + 1][j + 1]),
            };
        }
    }
    matches[0][0]
}

fn is_line_terminator(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
}
