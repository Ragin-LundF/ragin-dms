//! The parts of `dms-catalog.yaml`, as the Kotlin DTOs bound them. Every part with an identity or a singleton
//! keeps the fields only a newer version knows in `unknown`, so saving does not drop them.

mod catalog_dto;
mod category_dto;
mod classification_dto;
mod document_dto;
mod field_value_dto;
mod link_dto;
mod retention_dto;
pub mod rule;
pub mod settings;

pub use catalog_dto::CatalogDto;
pub use category_dto::CategoryDto;
pub use classification_dto::ClassificationDto;
pub use document_dto::DocumentDto;
pub use field_value_dto::FieldValueDto;
pub use link_dto::LinkDto;
pub use retention_dto::RetentionDto;

use indexmap::IndexMap;

use crate::yaml::{Fields, Node, YamlError};

/// A part of the catalog file that keeps the fields only a newer version knows.
pub trait ExtensibleDto {
    fn unknown(&self) -> &IndexMap<String, Node>;

    fn unknown_mut(&mut self) -> &mut IndexMap<String, Node>;
}

/// The items of the list property `key`, each read by `read`.
pub(crate) fn read_list<T>(
    fields: Fields<'_>,
    key: &str,
    read: impl Fn(&Node) -> Result<T, YamlError>,
) -> Result<Vec<T>, YamlError> {
    fields.list(key)?.iter().map(read).collect()
}

pub(crate) fn owned_texts(fields: Fields<'_>, key: &str) -> Result<Vec<String>, YamlError> {
    Ok(fields.texts(key)?.into_iter().map(str::to_owned).collect())
}

pub(crate) fn owned(text: Option<&str>) -> Option<String> {
    text.map(str::to_owned)
}

pub(crate) fn texts_node(texts: &[String]) -> Vec<Node> {
    texts.iter().map(Node::text).collect()
}

/// Implements [`ExtensibleDto`] for a DTO with an `unknown` field.
macro_rules! extensible {
    ($dto:ty) => {
        impl $crate::catalog::dto::ExtensibleDto for $dto {
            fn unknown(&self) -> &indexmap::IndexMap<String, $crate::yaml::Node> {
                &self.unknown
            }

            fn unknown_mut(&mut self) -> &mut indexmap::IndexMap<String, $crate::yaml::Node> {
                &mut self.unknown
            }
        }
    };
}

pub(crate) use extensible;
