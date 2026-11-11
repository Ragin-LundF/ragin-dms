use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::catalog::Catalog;
use crate::document::Document;
use crate::field::{FieldKey, FieldValues};
use crate::library::{FileNamePolicy, RelativePath};
use crate::restructure::{PlaceholderName, TemplateParse, TemplatePart, TemplateProblem, TemplateRendering};
use crate::text;

const FIELD_PREFIX: &str = "field:";

/// Where a document belongs, written like `{category}/{year|Undated}/`: folder names separated by `/`, with
/// placeholders in braces and an optional fallback after `|`. Every rendered folder name is made valid with
/// [`FileNamePolicy::sanitize`], so a value never breaks the path. Two templates are equal when their text is.
#[derive(Debug, Clone)]
pub struct PathTemplate {
    text: String,
    segments: Vec<Vec<TemplatePart>>,
}

impl PathTemplate {
    /// Reads `raw`; surrounding blanks and a leading or trailing `/` are ignored.
    #[must_use]
    pub fn parse(raw: &str) -> TemplateParse {
        let text = text::trim(raw).trim_matches('/');
        if text.is_empty() {
            return TemplateParse::Invalid(TemplateProblem::Empty);
        }
        let mut segments = Vec::new();
        for segment in text.split('/') {
            match parse_segment(text::trim(segment)) {
                Ok(parts) => segments.push(parts),
                Err(problem) => return TemplateParse::Invalid(problem),
            }
        }
        TemplateParse::Parsed(Self { text: text.to_owned(), segments })
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The placeholders the template uses, for previews and skip reasons.
    #[must_use]
    pub fn placeholders(&self) -> Vec<&TemplatePart> {
        self.segments.iter().flatten().filter(|part| matches!(part, TemplatePart::Placeholder { .. })).collect()
    }

    /// The folder `document` belongs in, or the placeholder it has no value for.
    ///
    /// # Panics
    /// Never: sanitized folder names always make a valid path.
    #[must_use]
    pub fn render(&self, document: &Document, catalog: &Catalog) -> TemplateRendering {
        let mut folders = Vec::new();
        for parts in &self.segments {
            let mut pieces = Vec::with_capacity(parts.len());
            for part in parts {
                match part {
                    TemplatePart::Literal(literal) => pieces.push(vec![literal.clone()]),
                    TemplatePart::Placeholder { name, field_key, fallback } => {
                        match value_of(*name, field_key.as_ref(), fallback.as_deref(), document, catalog) {
                            Some(value) => pieces.push(value),
                            None => return TemplateRendering::Missing(*name),
                        }
                    }
                }
            }
            folders.extend(join(pieces));
        }
        let names: Vec<String> = folders
            .iter()
            .filter(|folder| !text::is_blank(folder))
            .map(|folder| FileNamePolicy::sanitize(folder))
            .collect();
        TemplateRendering::Rendered(
            RelativePath::from_segments(&names).expect("invariant: sanitized names make a valid path"),
        )
    }
}

impl PartialEq for PathTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for PathTemplate {}

impl Hash for PathTemplate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl fmt::Display for PathTemplate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

/// Joins the pieces of one template segment. A piece with several folder names (a category path) contributes
/// its first names as folders of their own and its last to the segment's text.
fn join(pieces: Vec<Vec<String>>) -> Vec<String> {
    let mut result = vec![String::new()];
    for piece in pieces {
        let mut names = piece.into_iter();
        if let (Some(first), Some(last)) = (names.next(), result.last_mut()) {
            last.push_str(&first);
        }
        result.extend(names);
    }
    result
}

/// The value of a placeholder as one or more folder names, or `None` if there is none.
fn value_of(
    name: PlaceholderName,
    field_key: Option<&FieldKey>,
    fallback: Option<&str>,
    document: &Document,
    catalog: &Catalog,
) -> Option<Vec<String>> {
    let date = document.metadata.document_date;
    let value = match name {
        PlaceholderName::Category => category_path(document, catalog),
        PlaceholderName::CategoryTop => category_path(document, catalog).map(|mut path| {
            path.truncate(1);
            path
        }),
        PlaceholderName::Year => date.map(|date| vec![chrono::Datelike::year(&date).to_string()]),
        PlaceholderName::Month => date.map(|date| vec![format!("{:02}", chrono::Datelike::month(&date))]),
        PlaceholderName::Correspondent => document.metadata.correspondent.clone().map(|name| vec![name]),
        PlaceholderName::Kind => Some(vec![kind_name(document)]),
        PlaceholderName::Field => {
            field_key.and_then(|key| document.fields.get(key)).map(|value| vec![FieldValues::format(value)])
        }
    };
    match value {
        Some(names) => Some(names.iter().map(|name| FileNamePolicy::sanitize(name)).collect()),
        None => fallback.map(|fallback| vec![fallback.to_owned()]),
    }
}

/// `PDF` becomes `Pdf`.
fn kind_name(document: &Document) -> String {
    let lowered = text::lowercase(document.kind().name());
    let mut characters = lowered.chars();
    characters.next().map(|first| first.to_uppercase().chain(characters).collect()).unwrap_or_default()
}

fn category_path(document: &Document, catalog: &Catalog) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let mut visited = HashSet::new();
    let mut current = document.category_id.as_ref().and_then(|id| catalog.categories().get(id));
    while let Some(category) = current {
        if !visited.insert(&category.id) {
            break;
        }
        names.insert(0, category.name.clone());
        current = category.parent_id.as_ref().and_then(|id| catalog.categories().get(id));
    }
    (!names.is_empty()).then_some(names)
}

fn parse_segment(segment: &str) -> Result<Vec<TemplatePart>, TemplateProblem> {
    let mut parts = Vec::new();
    let mut rest = segment;
    while !rest.is_empty() {
        let Some(open) = rest.find('{') else {
            parts.push(TemplatePart::Literal(rest.to_owned()));
            break;
        };
        if open > 0 {
            parts.push(TemplatePart::Literal(rest[..open].to_owned()));
        }
        let close = rest[open..].find('}').map(|offset| open + offset).ok_or(TemplateProblem::UnclosedBrace)?;
        parts.push(placeholder_of(&rest[open + 1..close])?);
        rest = &rest[close + 1..];
    }
    let literal: String = parts
        .iter()
        .filter_map(|part| match part {
            TemplatePart::Literal(text) => Some(text.as_str()),
            TemplatePart::Placeholder { .. } => None,
        })
        .collect();
    if parts.len() == 1 && !literal.is_empty() && FileNamePolicy::check(&literal).is_some() {
        return Err(TemplateProblem::InvalidFolderName);
    }
    if literal.chars().any(|character| "<>:\"\\|?*".contains(character)) {
        return Err(TemplateProblem::InvalidFolderName);
    }
    Ok(parts)
}

fn placeholder_of(body: &str) -> Result<TemplatePart, TemplateProblem> {
    let (token, fallback) = body.split_once('|').unwrap_or((body, ""));
    let token = text::trim(token);
    let fallback = Some(text::trim(fallback)).filter(|fallback| !fallback.is_empty()).map(str::to_owned);
    if let Some(key) = token.strip_prefix(FIELD_PREFIX) {
        // Kotlin throws for a well-formed key longer than 40 characters; here it is an invalid key as well.
        let field_key = FieldKey::new(key).map_err(|_| TemplateProblem::InvalidFieldKey)?;
        return Ok(TemplatePart::Placeholder { name: PlaceholderName::Field, field_key: Some(field_key), fallback });
    }
    match PlaceholderName::of(token) {
        None | Some(PlaceholderName::Field) => Err(TemplateProblem::UnknownPlaceholder),
        Some(name) => Ok(TemplatePart::Placeholder { name, field_key: None, fallback }),
    }
}
