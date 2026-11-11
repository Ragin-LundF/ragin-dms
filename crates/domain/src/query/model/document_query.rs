use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};

use crate::DomainError;
use crate::catalog::Catalog;
use crate::document::Document;
use crate::error::require;
use crate::library::RelativePath;
use crate::query::{DateRange, FieldCondition, QueryContext, TagFilter};
use crate::taxonomy::{CategoryId, TagName};
use crate::text;

/// Which documents to show, export, restructure or ask about. Every condition that is set must hold; an empty
/// query (the [`Default`]) matches every document.
///
/// [`Self::text`] looks at the path, title, correspondent, tags and note. [`Self::category`] includes its
/// subcategories. [`Self::dates`] applies to the document date, or to the day the document was added when it has
/// none. [`Self::content_text`] is a term in the documents' text; [`Self::matches`] ignores it, because only the
/// content search can read the texts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentQuery {
    pub text: Option<String>,
    pub category: Option<CategoryId>,
    pub uncategorized: bool,
    pub tags: TagFilter,
    pub untagged: bool,
    pub dates: Option<DateRange>,
    pub correspondent: Option<String>,
    pub folder: Option<RelativePath>,
    pub fields: Vec<FieldCondition>,
    pub added_since: Option<DateTime<Utc>>,
    pub content_text: Option<String>,
}

impl DocumentQuery {
    /// # Errors
    /// Fails for a query that asks for a category and none at once, or with an invalid date range or field
    /// condition.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!(self.uncategorized && self.category.is_some()), || {
            "A query cannot ask for a category and none".to_owned()
        })?;
        if let Some(dates) = &self.dates {
            dates.validate()?;
        }
        self.fields.iter().try_for_each(FieldCondition::validate)
    }

    /// `true` if the query needs the documents' text through the content search.
    #[must_use]
    pub fn needs_content(&self) -> bool {
        self.content_text.as_deref().is_some_and(|content| !text::is_blank(content))
    }

    #[must_use]
    pub fn matches(&self, document: &Document, context: &QueryContext<'_>) -> bool {
        self.matches_text(document)
            && self.matches_category(document, context.catalog)
            && self.matches_tags(document)
            && self.dates.is_none_or(|dates| dates.contains(date_of(document, context)))
            && self.matches_correspondent(document)
            && self.folder.as_ref().is_none_or(|folder| document.path.is_inside(folder))
            && self.fields.iter().all(|condition| condition.matches(&document.fields, context.today))
            && self.added_since.is_none_or(|since| document.added_at >= since)
    }

    /// `true` if the query names a category, tag or field the catalog no longer has, for example after the
    /// category was removed; such a smart folder is shown as invalid instead of silently empty.
    #[must_use]
    pub fn refers_to_missing(&self, catalog: &Catalog) -> bool {
        let mut tags_used = self.tags.all.iter().chain(&self.tags.any).chain(&self.tags.none);
        self.category.as_ref().is_some_and(|category| !catalog.categories().contains_key(category))
            || !tags_used.all(|tag| catalog.tags().contains(tag))
            || self.fields.iter().any(|condition| catalog.settings().field(&condition.key).is_none())
    }

    /// The documents of `context`'s catalog this query matches, by path.
    #[must_use]
    pub fn filter<'a>(&self, context: &QueryContext<'a>) -> Vec<&'a Document> {
        let mut matching: Vec<&Document> =
            context.catalog.documents().values().filter(|document| self.matches(document, context)).collect();
        matching.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        matching
    }

    fn matches_text(&self, document: &Document) -> bool {
        let Some(wanted) = self.text.as_deref().map(|wanted| text::lowercase(text::trim(wanted))) else {
            return true;
        };
        if wanted.is_empty() {
            return true;
        }
        let metadata = &document.metadata;
        [
            Some(document.path.value()),
            metadata.title.as_deref(),
            metadata.correspondent.as_deref(),
            document.note.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(document.tags.iter().map(TagName::value))
        .any(|haystack| text::lowercase(haystack).contains(&wanted))
    }

    fn matches_category(&self, document: &Document, catalog: &Catalog) -> bool {
        if self.uncategorized {
            return document.category_id.is_none();
        }
        let Some(wanted) = &self.category else {
            return true;
        };
        let mut current = document.category_id.as_ref();
        let mut visited = HashSet::new();
        while let Some(category) = current {
            if !visited.insert(category) {
                break;
            }
            if category == wanted {
                return true;
            }
            current = catalog.categories().get(category).and_then(|it| it.parent_id.as_ref());
        }
        false
    }

    fn matches_tags(&self, document: &Document) -> bool {
        if self.untagged && !document.tags.is_empty() {
            return false;
        }
        self.tags.matches(&document.tags)
    }

    fn matches_correspondent(&self, document: &Document) -> bool {
        let Some(wanted) = self.correspondent.as_deref().map(|wanted| text::lowercase(text::trim(wanted))) else {
            return true;
        };
        if wanted.is_empty() {
            return true;
        }
        document.metadata.correspondent.as_deref().map(|name| text::lowercase(text::trim(name))) == Some(wanted)
    }
}

fn date_of(document: &Document, context: &QueryContext<'_>) -> NaiveDate {
    document.metadata.document_date.unwrap_or_else(|| document.added_at.with_timezone(&context.zone).date_naive())
}
