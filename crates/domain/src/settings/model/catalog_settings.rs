use std::collections::HashSet;
use std::hash::Hash;

use chrono::NaiveDate;

use crate::DomainError;
use crate::emergency::EmergencyFolder;
use crate::error::require;
use crate::expectation::ExpectationOverride;
use crate::field::{FieldDefinition, FieldKey};
use crate::query::{SavedSearch, SavedSearchId};
use crate::rule::{Correspondent, MatchingRule, MatchingRuleId};
use crate::settings::{LibraryLayout, OwnerDetails};

/// The library-wide settings that travel with the catalog to every folder and device: custom field definitions,
/// saved searches, matching rules, known correspondents and the [`Self::layout`], plus the owner's details, the day
/// bills are tracked since, dismissed contract proposals (sender names, compared ignoring case), the decisions
/// about expected documents and the emergency folder.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CatalogSettings {
    pub field_definitions: Vec<FieldDefinition>,
    pub saved_searches: Vec<SavedSearch>,
    pub rules: Vec<MatchingRule>,
    pub correspondents: Vec<Correspondent>,
    pub layout: LibraryLayout,
    pub owner: Option<OwnerDetails>,
    pub bills_tracked_since: Option<NaiveDate>,
    pub dismissed_contract_proposals: Vec<String>,
    pub expectations: Vec<ExpectationOverride>,
    pub emergency_folder: Option<EmergencyFolder>,
}

impl CatalogSettings {
    /// # Errors
    /// Fails for two entries with the same key, id or name, or for an invalid entry.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(all_distinct(self.field_definitions.iter().map(|field| &field.key)), || {
            "Two field definitions share a key".to_owned()
        })?;
        require(all_distinct(self.saved_searches.iter().map(|search| &search.id)), || {
            "Two saved searches share an id".to_owned()
        })?;
        require(all_distinct(self.rules.iter().map(|rule| &rule.id)), || "Two rules share an id".to_owned())?;
        require(all_distinct(self.correspondents.iter().map(Correspondent::key)), || {
            "Two correspondents share a name".to_owned()
        })?;
        self.field_definitions.iter().try_for_each(FieldDefinition::validate)?;
        self.saved_searches.iter().try_for_each(SavedSearch::validate)?;
        self.rules.iter().try_for_each(MatchingRule::validate)?;
        self.correspondents.iter().try_for_each(Correspondent::validate)?;
        require(all_distinct(self.expectations.iter().map(ExpectationOverride::key)), || {
            "Two expectations share a correspondent and category".to_owned()
        })?;
        self.expectations.iter().try_for_each(ExpectationOverride::validate)?;
        require(self.dismissed_contract_proposals.iter().all(|name| !crate::text::is_blank(name)), || {
            "A dismissed contract proposal needs a correspondent".to_owned()
        })?;
        self.owner.as_ref().map_or(Ok(()), OwnerDetails::validate)?;
        self.emergency_folder.as_ref().map_or(Ok(()), EmergencyFolder::validate)?;
        self.layout.validate()
    }

    #[must_use]
    pub fn field(&self, key: &FieldKey) -> Option<&FieldDefinition> {
        self.field_definitions.iter().find(|definition| &definition.key == key)
    }

    /// Adds `definition`, or replaces the definition with the same key in place.
    #[must_use]
    pub fn with_field(&self, definition: FieldDefinition) -> Self {
        let key = definition.key.clone();
        Self { field_definitions: replaced(&self.field_definitions, definition, |it| it.key == key), ..self.clone() }
    }

    #[must_use]
    pub fn without_field(&self, key: &FieldKey) -> Self {
        let field_definitions = self.field_definitions.iter().filter(|it| &it.key != key).cloned().collect();
        Self { field_definitions, ..self.clone() }
    }

    /// Adds `search`, or replaces the saved search with the same id in place.
    #[must_use]
    pub fn with_search(&self, search: SavedSearch) -> Self {
        let id = search.id.clone();
        Self { saved_searches: replaced(&self.saved_searches, search, |it| it.id == id), ..self.clone() }
    }

    #[must_use]
    pub fn without_search(&self, id: &SavedSearchId) -> Self {
        Self { saved_searches: self.saved_searches.iter().filter(|it| &it.id != id).cloned().collect(), ..self.clone() }
    }

    /// Adds `rule`, or replaces the rule with the same id in place.
    #[must_use]
    pub fn with_rule(&self, rule: MatchingRule) -> Self {
        let id = rule.id.clone();
        Self { rules: replaced(&self.rules, rule, |it| it.id == id), ..self.clone() }
    }

    #[must_use]
    pub fn without_rule(&self, id: &MatchingRuleId) -> Self {
        Self { rules: self.rules.iter().filter(|it| &it.id != id).cloned().collect(), ..self.clone() }
    }

    /// Adds `correspondent`, or replaces the one with the same name (ignoring case) in place.
    #[must_use]
    pub fn with_correspondent(&self, correspondent: Correspondent) -> Self {
        let key = correspondent.key();
        Self { correspondents: replaced(&self.correspondents, correspondent, |it| it.key() == key), ..self.clone() }
    }

    #[must_use]
    pub fn with_layout(&self, layout: LibraryLayout) -> Self {
        Self { layout, ..self.clone() }
    }
}

fn replaced<T: Clone>(list: &[T], item: T, same: impl Fn(&T) -> bool) -> Vec<T> {
    if !list.iter().any(&same) {
        let mut extended = list.to_vec();
        extended.push(item);
        return extended;
    }
    list.iter().map(|it| if same(it) { item.clone() } else { it.clone() }).collect()
}

fn all_distinct<T: Eq + Hash>(items: impl ExactSizeIterator<Item = T>) -> bool {
    let count = items.len();
    items.collect::<HashSet<_>>().len() == count
}
