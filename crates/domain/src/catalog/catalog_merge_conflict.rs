use crate::document::Document;
use crate::field::FieldDefinition;
use crate::query::SavedSearch;
use crate::rule::MatchingRule;
use crate::taxonomy::Category;

/// Something [`crate::catalog::CatalogMerger`] had to decide between two catalog copies. The user reviews each one.
#[derive(Debug, Clone, PartialEq)]
pub enum CatalogMergeConflict {
    /// The same document differs in both copies; the newer copy's version was kept.
    DocumentDiffers { kept: Document, discarded: Document },
    /// Two different documents claim the same path; the newer copy's document was kept.
    PathClash { kept: Document, discarded: Document },
    /// The same category differs in both copies; the newer copy's version was kept.
    CategoryDiffers { kept: Category, discarded: Category },
    /// The same custom field is defined differently in both copies; the newer copy's definition was kept.
    FieldDefinitionDiffers { kept: FieldDefinition, discarded: FieldDefinition },
    /// The same saved search differs in both copies; the newer copy's version was kept.
    SavedSearchDiffers { kept: SavedSearch, discarded: SavedSearch },
    /// The same matching rule differs in both copies; the newer copy's version was kept.
    RuleDiffers { kept: MatchingRule, discarded: MatchingRule },
}
