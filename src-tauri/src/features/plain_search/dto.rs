use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::library::dto::params::LibraryFilterDto;

/// A sentence as filters: the facet filters to add (payment state and tax year among them), the words left for the
/// search field, and what the library does not know.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlainSearchResultDto {
    pub filters: Vec<LibraryFilterDto>,
    pub text: String,
    pub not_understood: Vec<String>,
}
