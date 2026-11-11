use crate::DomainError;
use crate::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use crate::taxonomy::{Category, CategoryId, TagName};
use crate::text;

use super::catalog::is_in_cycle;

/// The changes to categories and tags.
impl Catalog {
    /// # Errors
    /// Fails for an invalid category.
    pub fn add_category(&self, category: Category) -> Result<CatalogUpdate, DomainError> {
        category.validate()?;
        if self.categories.contains_key(&category.id) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryAlreadyExists));
        }
        if category.parent_id.as_ref().is_some_and(|parent| !self.categories.contains_key(parent)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound));
        }
        if self.is_name_taken(&category.name, category.parent_id.as_ref(), None) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNameTaken));
        }
        Ok(self.applied(|catalog| {
            catalog.categories.insert(category.id.clone(), category);
        }))
    }

    /// # Errors
    /// Fails for a blank name.
    pub fn rename_category(&self, category_id: &CategoryId, name: &str) -> Result<CatalogUpdate, DomainError> {
        let Some(category) = self.categories.get(category_id) else {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound));
        };
        let renamed = Category { name: name.to_owned(), ..category.clone() };
        renamed.validate()?;
        if self.is_name_taken(name, category.parent_id.as_ref(), Some(category_id)) {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNameTaken));
        }
        Ok(self.applied(|catalog| {
            catalog.categories.insert(category_id.clone(), renamed);
        }))
    }

    /// Moves a category below `parent_id`, or to the top level with `None`.
    #[must_use]
    pub fn move_category(&self, category_id: &CategoryId, parent_id: Option<&CategoryId>) -> CatalogUpdate {
        let Some(category) = self.categories.get(category_id) else {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound);
        };
        if parent_id.is_some_and(|parent| !self.categories.contains_key(parent)) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound);
        }
        if parent_id == Some(category_id) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryCycle);
        }
        let mut moved = self.categories.clone();
        moved.insert(category_id.clone(), Category { parent_id: parent_id.cloned(), ..category.clone() });
        if is_in_cycle(category_id, &moved) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryCycle);
        }
        if self.is_name_taken(&category.name, parent_id, Some(category_id)) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryNameTaken);
        }
        self.applied(|catalog| catalog.categories = moved)
    }

    /// Sets the opt-in retention period of a category in years; `None` removes it.
    ///
    /// # Errors
    /// Fails for a period outside 1..=100 years.
    pub fn set_retention_years(
        &self,
        category_id: &CategoryId,
        years: Option<u32>,
    ) -> Result<CatalogUpdate, DomainError> {
        let Some(category) = self.categories.get(category_id) else {
            return Ok(CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound));
        };
        let changed = Category { retention_years: years, ..category.clone() };
        changed.validate()?;
        Ok(self.applied(|catalog| {
            catalog.categories.insert(category_id.clone(), changed);
        }))
    }

    /// Removes an unused category without children.
    #[must_use]
    pub fn remove_category(&self, category_id: &CategoryId) -> CatalogUpdate {
        if !self.categories.contains_key(category_id) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryNotFound);
        }
        if self.documents.values().any(|document| document.category_id.as_ref() == Some(category_id)) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryInUse);
        }
        if self.categories.values().any(|category| category.parent_id.as_ref() == Some(category_id)) {
            return CatalogUpdate::Rejected(CatalogRejection::CategoryHasChildren);
        }
        self.applied(|catalog| {
            catalog.categories.shift_remove(category_id);
        })
    }

    /// Renames `from` to `to` everywhere. If `to` already exists, the two tags are merged. Renaming to a different
    /// casing of the same tag changes how it is displayed.
    #[must_use]
    pub fn rename_tag(&self, from: &TagName, to: TagName) -> CatalogUpdate {
        if !self.tags.contains(from) {
            return CatalogUpdate::Rejected(CatalogRejection::TagNotFound);
        }
        let mut remaining = self.tags.clone();
        remaining.shift_remove(from);
        // Merging into an existing tag keeps that tag's spelling.
        let target = remaining.get(&to).cloned().unwrap_or(to);
        self.applied(|catalog| {
            for document in catalog.documents.values_mut() {
                if document.tags.shift_remove(from) {
                    document.tags.insert(target.clone());
                }
            }
            remaining.insert(target);
            catalog.tags = remaining;
        })
    }

    /// Removes a tag from the catalog and from every document.
    #[must_use]
    pub fn remove_tag(&self, tag: &TagName) -> CatalogUpdate {
        if !self.tags.contains(tag) {
            return CatalogUpdate::Rejected(CatalogRejection::TagNotFound);
        }
        self.applied(|catalog| {
            for document in catalog.documents.values_mut() {
                document.tags.shift_remove(tag);
            }
            catalog.tags.shift_remove(tag);
        })
    }

    fn is_name_taken(&self, name: &str, parent_id: Option<&CategoryId>, except: Option<&CategoryId>) -> bool {
        let key = text::lowercase(text::trim(name));
        self.categories.values().any(|category| {
            Some(&category.id) != except
                && category.parent_id.as_ref() == parent_id
                && text::lowercase(text::trim(&category.name)) == key
        })
    }
}
