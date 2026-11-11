//! The dashboard's figures from the catalog: stat strip, AI band, charts and the recently added documents
//! (Kotlin `LibraryStatistics`, `StatColumns`, `AiBandContent`, `DashboardHeader`).

use std::cmp::Reverse;
use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentKind};
use dms_domain::text;

use crate::features::dashboard::dto::{
    AiBandDto, ChartPointDto, DocumentKindDto, KindCountDto, RecentDocumentDto, StatColumnDto, StatLink,
};
use crate::features::dashboard::format;

const YEARS: i32 = 8;
const TOP_CATEGORIES: usize = 8;
const TOP_TAGS: usize = 24;
const TOP_CORRESPONDENTS: usize = 6;
const RECENT: usize = 6;
const GROWTH_DAYS: i64 = 30;

/// The inbox while it is switched on: its path, the documents in it, and whether the AI analyzes them.
pub struct InboxSummary {
    pub path: String,
    pub count: usize,
    pub ai_analysis: bool,
}

pub fn inbox_of(catalog: &Catalog, ai_enabled: bool) -> Option<InboxSummary> {
    let inbox = &catalog.settings().layout.inbox;
    if !inbox.enabled {
        return None;
    }
    Some(InboxSummary {
        path: inbox.path.to_string(),
        count: catalog.documents().values().filter(|document| inbox.contains(&document.path)).count(),
        ai_analysis: inbox.ai_analysis && ai_enabled,
    })
}

/// Documents added after `now − 30 days`.
pub fn added_recently(catalog: &Catalog, now: DateTime<Utc>) -> usize {
    let since = now - Duration::days(GROWTH_DAYS);
    catalog.documents().values().filter(|document| document.added_at > since).count()
}

/// The size of the library at each of the last 30 days, oldest first; day N counts what was added up to
/// `now − N × 24 h`.
pub fn growth(catalog: &Catalog, now: DateTime<Utc>) -> Vec<u32> {
    if catalog.documents().is_empty() {
        return Vec::new();
    }
    (0..GROWTH_DAYS)
        .rev()
        .map(|days_ago| {
            let end = now - Duration::days(days_ago);
            count(catalog.documents().values().filter(|document| document.added_at <= end).count())
        })
        .collect()
}

/// The four figures: documents, categorized, inbox, storage.
pub fn stat_columns(
    catalog: &Catalog,
    inbox: Option<&InboxSummary>,
    folder_count: usize,
    by_year: &[ChartPointDto],
    now: DateTime<Utc>,
) -> Vec<StatColumnDto> {
    let documents = catalog.documents().len();
    let recently = added_recently(catalog, now);
    let uncategorized = uncategorized(catalog);
    vec![
        StatColumnDto {
            label: "Documents".to_owned(),
            value: documents.to_string(),
            count: Some(count(documents)),
            caption: if recently > 0 {
                format!("+{recently} in the last 30 days")
            } else {
                "None added lately".to_owned()
            },
            meter: None,
            // A month without additions draws a flat line, so the years tell more then.
            trend: if recently > 0 { growth(catalog, now) } else { by_year.iter().map(|point| point.value).collect() },
            link: None,
        },
        categorized(documents, uncategorized),
        inbox_column(inbox),
        StatColumnDto {
            label: "Storage".to_owned(),
            value: format::file_size(total_bytes(catalog)),
            count: None,
            caption: format!("Kept in each of {folder_count} library folders"),
            meter: None,
            trend: Vec::new(),
            link: None,
        },
    ]
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)] // a share in percent
fn categorized(documents: usize, uncategorized: usize) -> StatColumnDto {
    let share = if documents == 0 { 0.0 } else { (documents - uncategorized) as f32 / documents as f32 };
    StatColumnDto {
        label: "Categorized".to_owned(),
        value: format!("{} %", (share * 100.0).round() as u32),
        count: None,
        caption: format!("{uncategorized} without a category"),
        meter: Some(share),
        trend: Vec::new(),
        link: (uncategorized > 0).then_some(StatLink::Uncategorized),
    }
}

fn inbox_column(inbox: Option<&InboxSummary>) -> StatColumnDto {
    let (value, caption, link) = match inbox {
        None => ("Off".to_owned(), "Switched off in the settings", None),
        Some(inbox) => (
            inbox.count.to_string(),
            if inbox.count == 0 { "Nothing to file" } else { "Waiting to be filed" },
            Some(StatLink::Inbox),
        ),
    };
    StatColumnDto {
        label: "Inbox".to_owned(),
        value,
        count: None,
        caption: caption.to_owned(),
        meter: None,
        trend: Vec::new(),
        link,
    }
}

/// The AI band, or `None` while AI is off or neither run has work. Candidates are documents the AI can read with
/// no suggestion waiting.
pub fn ai_band(ai_enabled: bool, untagged: usize, unclassified: usize) -> Option<AiBandDto> {
    if !ai_enabled || untagged == 0 && unclassified == 0 {
        return None;
    }
    let no_tags = format!("{} no tags yet", has(untagged));
    let no_category = format!("{} no category", has(unclassified));
    let headline = if unclassified == 0 {
        format!("{} {no_tags}", documents(untagged))
    } else if untagged == 0 {
        format!("{} {no_category}", documents(unclassified))
    } else {
        format!("{} {no_tags}, {unclassified} {no_category}", documents(untagged))
    };
    Some(AiBandDto {
        headline,
        classify_label: (unclassified > 0).then(|| format!("Classify {unclassified}")),
        tag_label: (untagged > 0).then(|| format!("Tag {untagged} with AI")),
    })
}

fn documents(count: usize) -> String {
    if count == 1 { "1 document".to_owned() } else { format!("{count} documents") }
}

fn has(count: usize) -> &'static str {
    if count == 1 { "has" } else { "have" }
}

/// "{n} documents · {size} · {folders} library folders · last scan {dd.MM. HH:mm}" (no singular forms).
pub fn header_line<Tz: TimeZone>(
    catalog: &Catalog,
    folder_count: usize,
    last_scan_at: Option<DateTime<Utc>>,
    zone: &Tz,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let scanned = last_scan_at
        .map_or_else(|| "not scanned yet".to_owned(), |at| format!("last scan {}", format::day_and_time(at, zone)));
    format!(
        "{} documents · {} · {folder_count} library folders · {scanned}",
        catalog.documents().len(),
        format::file_size(total_bytes(catalog))
    )
}

pub fn uncategorized(catalog: &Catalog) -> usize {
    catalog.documents().values().filter(|document| document.category_id.is_none()).count()
}

fn total_bytes(catalog: &Catalog) -> u64 {
    catalog.documents().values().map(|document| document.size).sum()
}

pub fn kind_dto(kind: DocumentKind) -> DocumentKindDto {
    match kind {
        DocumentKind::Pdf => DocumentKindDto::Pdf,
        DocumentKind::Image => DocumentKindDto::Image,
        DocumentKind::Office => DocumentKindDto::Office,
        DocumentKind::Other => DocumentKindDto::Other,
    }
}

pub fn by_kind(catalog: &Catalog) -> Vec<KindCountDto> {
    [DocumentKind::Pdf, DocumentKind::Image, DocumentKind::Office, DocumentKind::Other]
        .into_iter()
        .map(|kind| KindCountDto {
            kind: kind_dto(kind),
            count: count(catalog.documents().values().filter(|document| document.kind() == kind).count()),
        })
        .filter(|entry| entry.count > 0)
        .collect()
}

/// The largest categories by the document's own category, then "Uncategorized" (id `""`) when there are any.
pub fn categories(catalog: &Catalog) -> Vec<ChartPointDto> {
    let names = catalog
        .documents()
        .values()
        .filter_map(|document| document.category_id.as_ref().and_then(|id| catalog.categories().get(id)))
        .map(|category| (category.id.value(), category.name.as_str()));
    let mut ranked = ranked(tally(names), TOP_CATEGORIES);
    let uncategorized = uncategorized(catalog);
    if uncategorized > 0 {
        ranked.push(point("", "Uncategorized", uncategorized));
    }
    ranked
}

pub fn tags(catalog: &Catalog) -> Vec<ChartPointDto> {
    let tags =
        catalog.documents().values().flat_map(|document| document.tags.iter()).map(|tag| (tag.value(), tag.value()));
    ranked(tally(tags), TOP_TAGS)
}

/// Grouped by the exact name, case-sensitive.
pub fn correspondents(catalog: &Catalog) -> Vec<ChartPointDto> {
    let names = catalog
        .documents()
        .values()
        .filter_map(|document| document.metadata.correspondent.as_deref())
        .map(|name| (name, name));
    ranked(tally(names), TOP_CORRESPONDENTS)
}

/// Counts `(id, label)` pairs by id, in the order the ids first appear (like Kotlin's `groupingBy`).
fn tally<'a>(entries: impl Iterator<Item = (&'a str, &'a str)>) -> Vec<ChartPointDto> {
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut points: Vec<ChartPointDto> = Vec::new();
    for (id, label) in entries {
        if let Some(&at) = index.get(id) {
            points[at].value += 1;
        } else {
            index.insert(id, points.len());
            points.push(point(id, label, 1));
        }
    }
    points
}

/// Documents per year of their document date, else of when they were added in `zone`; up to eight years ending
/// at this year, the first bar "≤YYYY" holding every older year too.
pub fn by_year<Tz: TimeZone>(catalog: &Catalog, now: DateTime<Utc>, zone: &Tz) -> Vec<ChartPointDto> {
    if catalog.documents().is_empty() {
        return Vec::new();
    }
    let mut per_year: HashMap<i32, usize> = HashMap::new();
    for document in catalog.documents().values() {
        *per_year.entry(year_of(document, zone)).or_default() += 1;
    }
    let this_year = now.with_timezone(zone).year();
    let oldest = per_year.keys().copied().min().unwrap_or(this_year).min(this_year);
    let first = oldest.max(this_year - YEARS + 1);
    (first..=this_year)
        .map(|year| {
            let folded = year == first && oldest < first;
            let value = if folded {
                per_year.iter().filter(|(entry, _)| **entry <= year).map(|(_, value)| value).sum()
            } else {
                per_year.get(&year).copied().unwrap_or(0)
            };
            let label = if folded { format!("≤{year}") } else { year.to_string() };
            point(&label, &label, value)
        })
        .collect()
}

fn year_of<Tz: TimeZone>(document: &Document, zone: &Tz) -> i32 {
    document.metadata.document_date.map_or_else(|| document.added_at.with_timezone(zone).year(), |date| date.year())
}

pub fn recent<Tz: TimeZone>(catalog: &Catalog, now: DateTime<Utc>, zone: &Tz) -> Vec<RecentDocumentDto> {
    let mut documents: Vec<&Document> = catalog.documents().values().collect();
    documents.sort_by_key(|document| Reverse(document.added_at));
    documents
        .into_iter()
        .take(RECENT)
        .map(|document| {
            let folder = document.path.parent().map(|parent| parent.to_string()).filter(|parent| !parent.is_empty());
            let category = document.category_id.as_ref().and_then(|id| catalog.categories().get(id));
            let mut subtitle = folder.unwrap_or_else(|| "Top level".to_owned());
            if let Some(category) = category {
                subtitle = format!("{subtitle} · {}", category.name);
            }
            RecentDocumentDto {
                id: document.id.to_string(),
                path: document.path.to_string(),
                title: document.metadata.title.clone().unwrap_or_else(|| document.path.file_name().to_owned()),
                subtitle,
                kind: kind_dto(document.kind()),
                added: format::relative(document.added_at, now, zone),
                added_on: format::absolute(document.added_at, zone),
            }
        })
        .collect()
}

fn point(id: &str, label: &str, value: usize) -> ChartPointDto {
    ChartPointDto { id: id.to_owned(), label: label.to_owned(), value: count(value) }
}

/// Largest first, ties by the lower-case label, at most `limit`.
fn ranked(mut points: Vec<ChartPointDto>, limit: usize) -> Vec<ChartPointDto> {
    points.sort_by(|first, second| {
        second
            .value
            .cmp(&first.value)
            .then_with(|| text::compare(&text::lowercase(&first.label), &text::lowercase(&second.label)))
    });
    points.truncate(limit);
    points
}

pub fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
