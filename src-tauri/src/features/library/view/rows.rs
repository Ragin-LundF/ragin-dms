//! The documents as rows with their amount and due date, their order and groups, the category tiles and the
//! sidebar's counts (Kotlin `DocumentRows`, `DocumentOrder`, `DocumentGroups`).

use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId, DocumentKind};
use dms_domain::field::{Decimal, FieldPresets, FieldRole, FieldValue};
use dms_domain::taxonomy::CategoryId;
use dms_domain::text;

use super::trees;
use crate::features::library::model::{GroupKey, Moment, Sort, SortColumn};

const TILE_PAGES: usize = 3;
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// One document as the grid and the list show it.
#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub document: &'a Document,
    pub category_name: Option<&'a str>,
    pub amount: Option<&'a FieldValue>,
    pub due_date: Option<NaiveDate>,
}

pub struct Group<'a> {
    pub title: String,
    pub rows: Vec<Row<'a>>,
}

/// A top-level category as a tile above "All documents".
pub struct Tile<'a> {
    pub category_id: &'a CategoryId,
    pub name: &'a str,
    pub count: usize,
    pub kinds: Vec<DocumentKind>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub all: usize,
    pub inbox: usize,
    pub needs_review: usize,
    pub added_this_month: usize,
    pub folders: usize,
}

pub fn row_of<'a>(document: &'a Document, catalog: &'a Catalog) -> Row<'a> {
    Row {
        document,
        category_name: document
            .category_id
            .as_ref()
            .and_then(|id| catalog.categories().get(id))
            .map(|it| it.name.as_str()),
        amount: amount_of(document),
        due_date: due_date_of(document, catalog),
    }
}

/// The amount field, or else the first amount of money the document has.
fn amount_of(document: &Document) -> Option<&FieldValue> {
    let is_money = |value: &&FieldValue| matches!(value, FieldValue::Money { .. });
    document
        .fields
        .get(&FieldPresets::amount().key)
        .filter(is_money)
        .or_else(|| document.fields.values().find(is_money))
}

/// The first date of a field the library marks as a due date.
fn due_date_of(document: &Document, catalog: &Catalog) -> Option<NaiveDate> {
    catalog
        .settings()
        .field_definitions
        .iter()
        .filter(|definition| definition.role == Some(FieldRole::DueDate))
        .find_map(|definition| match document.fields.get(&definition.key) {
            Some(FieldValue::Date(date)) => Some(*date),
            _ => None,
        })
}

/// `rows` in the order of `sort`; rows without a value in its column stay last in either direction.
pub fn sorted(rows: Vec<Row<'_>>, sort: Option<Sort>) -> Vec<Row<'_>> {
    let Some(sort) = sort else {
        return rows;
    };
    let (mut with_value, without): (Vec<Row<'_>>, Vec<Row<'_>>) =
        rows.into_iter().partition(|row| has_value(row, sort.column));
    with_value.sort_by(|first, second| {
        let ascending = compare(first, second, sort.column);
        if sort.descending { ascending.reverse() } else { ascending }
    });
    with_value.extend(without);
    with_value
}

fn has_value(row: &Row<'_>, column: SortColumn) -> bool {
    let metadata = &row.document.metadata;
    match column {
        SortColumn::Name => true,
        SortColumn::Category => row.category_name.is_some(),
        SortColumn::Correspondent => metadata.correspondent.is_some(),
        SortColumn::Date => metadata.document_date.is_some(),
        SortColumn::Amount => row.amount.is_some(),
    }
}

fn compare(first: &Row<'_>, second: &Row<'_>, column: SortColumn) -> Ordering {
    let by_text =
        |of: &dyn Fn(&Row<'_>) -> String| text::compare(&text::lowercase(&of(first)), &text::lowercase(&of(second)));
    match column {
        SortColumn::Name => by_text(&|row| name_of(row.document).to_owned()),
        SortColumn::Category => by_text(&|row| row.category_name.unwrap_or_default().to_owned()),
        SortColumn::Correspondent => by_text(&|row| row.document.metadata.correspondent.clone().unwrap_or_default()),
        SortColumn::Date => first.document.metadata.document_date.cmp(&second.document.metadata.document_date),
        SortColumn::Amount => match (money(first), money(second)) {
            (Some(first), Some(second)) => first.compare_value(second),
            _ => Ordering::Equal,
        },
    }
}

fn money<'a>(row: &Row<'a>) -> Option<&'a Decimal> {
    match row.amount {
        Some(FieldValue::Money { amount, .. }) => Some(amount),
        _ => None,
    }
}

/// The title, or the file name for a document without one.
pub fn name_of(document: &Document) -> &str {
    document.metadata.title.as_deref().unwrap_or_else(|| document.path.file_name())
}

/// Groups `rows` under headers; inside a group the rows keep their order.
pub fn groups<'a>(rows: Vec<Row<'a>>, key: GroupKey, moment: &Moment, catalog: &Catalog) -> Vec<Group<'a>> {
    match key {
        GroupKey::Month => by_month(rows, moment),
        GroupKey::Category => by_name(rows, "Without a category", |document| top_category_of(document, catalog)),
        GroupKey::Sender => by_name(rows, "Unknown sender", |document| {
            document.metadata.correspondent.as_deref().map(|name| text::trim(name).to_owned())
        }),
    }
}

/// By month of the document date, or of when a document without one was added; newest first.
fn by_month<'a>(rows: Vec<Row<'a>>, moment: &Moment) -> Vec<Group<'a>> {
    let mut months: Vec<((i32, u32), Vec<Row<'a>>)> = Vec::new();
    for row in rows {
        let date = row
            .document
            .metadata
            .document_date
            .unwrap_or_else(|| row.document.added_at.with_timezone(&moment.zone).date_naive());
        let month = (date.year(), date.month());
        match months.iter_mut().find(|(key, _)| *key == month) {
            Some((_, grouped)) => grouped.push(row),
            None => months.push((month, vec![row])),
        }
    }
    months.sort_by_key(|(month, _)| std::cmp::Reverse(*month));
    months
        .into_iter()
        .map(|((year, month), rows)| Group { title: format!("{} {year}", MONTHS[month as usize - 1]), rows })
        .collect()
}

/// Groups by `name_of` ignoring case, sorted by the lowercase name, the first spelling as title; rows without a
/// name go to `missing`, last.
fn by_name<'a>(rows: Vec<Row<'a>>, missing: &str, name_of: impl Fn(&Document) -> Option<String>) -> Vec<Group<'a>> {
    let mut named: Vec<(String, Group<'a>)> = Vec::new();
    let mut unnamed = Vec::new();
    for row in rows {
        let Some(name) = name_of(row.document) else {
            unnamed.push(row);
            continue;
        };
        let key = text::lowercase(&name);
        match named.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, group)) => group.rows.push(row),
            None => named.push((key, Group { title: name, rows: vec![row] })),
        }
    }
    named.sort_by(|first, second| text::compare(&first.0, &second.0));
    let mut groups: Vec<Group<'a>> = named.into_iter().map(|(_, group)| group).collect();
    if !unnamed.is_empty() {
        groups.push(Group { title: missing.to_owned(), rows: unnamed });
    }
    groups
}

fn top_category_of(document: &Document, catalog: &Catalog) -> Option<String> {
    let mut current = catalog.categories().get(document.category_id.as_ref()?)?;
    let mut visited = HashSet::new();
    while visited.insert(&current.id) {
        match current.parent_id.as_ref().and_then(|parent| catalog.categories().get(parent)) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    Some(current.name.clone())
}

fn top_of<'a>(id: &'a CategoryId, catalog: &'a Catalog) -> &'a CategoryId {
    let mut visited = HashSet::new();
    let mut current = id;
    while visited.insert(current) {
        match catalog.categories().get(current).and_then(|category| category.parent_id.as_ref()) {
            Some(parent) => current = parent,
            None => return current,
        }
    }
    current
}

/// The top-level categories that hold any document, by name, each with its subcategories' documents.
pub fn tiles(catalog: &Catalog) -> Vec<Tile<'_>> {
    let mut by_top: Vec<(&CategoryId, Vec<&Document>)> = Vec::new();
    for document in catalog.documents().values() {
        let Some(id) = &document.category_id else {
            continue;
        };
        let top = top_of(id, catalog);
        match by_top.iter_mut().find(|(existing, _)| *existing == top) {
            Some((_, inside)) => inside.push(document),
            None => by_top.push((top, vec![document])),
        }
    }
    let mut tiles: Vec<(String, Tile<'_>)> = by_top
        .into_iter()
        .filter_map(|(top, mut inside)| {
            let category = catalog.categories().get(top)?;
            inside.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
            let tile = Tile {
                category_id: top,
                name: &category.name,
                count: inside.len(),
                kinds: inside.iter().take(TILE_PAGES).map(|document| document.kind()).collect(),
            };
            Some((text::lowercase(&category.name), tile))
        })
        .collect();
    tiles.sort_by(|first, second| text::compare(&first.0, &second.0));
    tiles.into_iter().map(|(_, tile)| tile).collect()
}

pub fn counts(catalog: &Catalog, review_ids: &HashSet<DocumentId>, moment: &Moment) -> Counts {
    let documents = catalog.documents();
    let inbox = &catalog.settings().layout.inbox;
    let month_start = moment.start_of_month();
    let folders: HashSet<_> = documents.values().flat_map(|document| trees::ancestors_of(&document.path)).collect();
    Counts {
        all: documents.len(),
        inbox: documents.values().filter(|document| inbox.contains(&document.path)).count(),
        needs_review: review_ids.iter().filter(|id| documents.contains_key(*id)).count(),
        added_this_month: documents.values().filter(|document| document.added_at >= month_start).count(),
        folders: folders.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::library::test_catalog::{TestCatalog, date, moment};
    use dms_domain::field::{FieldDefinition, FieldKey, FieldType};

    fn rows(catalog: &Catalog) -> Vec<Row<'_>> {
        catalog.documents().values().map(|document| row_of(document, catalog)).collect()
    }

    fn names(rows: &[Row<'_>]) -> Vec<String> {
        rows.iter().map(|row| row.document.path.value().to_owned()).collect()
    }

    #[test]
    fn rows_take_the_amount_field_or_the_first_money_and_the_due_date_field() {
        let catalog = TestCatalog::new()
            .settings(|settings| {
                settings.field_definitions.push(
                    FieldDefinition::new(
                        FieldKey::new("pay_by").expect("key"),
                        "Pay by",
                        FieldType::Date,
                        Some(FieldRole::DueDate),
                    )
                    .expect("definition"),
                );
            })
            .document("a.pdf", |document| {
                document.money("total", "12.50").money("amount", "99").field_date("pay_by", date(2025, 11, 1));
            })
            .document("b.pdf", |document| {
                document.money("total", "7");
            })
            .build();
        let rows = rows(&catalog);
        assert_eq!(rows[0].amount.map(dms_domain::field::FieldValues::format), Some("99 EUR".to_owned()));
        assert_eq!(rows[0].due_date, Some(date(2025, 11, 1)));
        assert_eq!(rows[1].amount.map(dms_domain::field::FieldValues::format), Some("7 EUR".to_owned()));
    }

    #[test]
    fn sorting_keeps_rows_without_a_value_last_in_both_directions() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.dated(date(2024, 1, 1));
            })
            .document("b.pdf", |_| {})
            .document("c.pdf", |document| {
                document.dated(date(2025, 1, 1));
            })
            .build();
        let ascending = Sort { column: SortColumn::Date, descending: false };
        assert_eq!(names(&sorted(rows(&catalog), Some(ascending))), ["a.pdf", "c.pdf", "b.pdf"]);
        let descending = Sort { descending: true, ..ascending };
        assert_eq!(names(&sorted(rows(&catalog), Some(descending))), ["c.pdf", "a.pdf", "b.pdf"]);
    }

    #[test]
    fn months_are_newest_first_with_english_titles() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.dated(date(2024, 10, 3));
            })
            .document("b.pdf", |document| {
                document.dated(date(2025, 2, 1));
            })
            .document("c.pdf", |document| {
                document.dated(date(2024, 10, 9));
            })
            .build();
        let groups = groups(rows(&catalog), GroupKey::Month, &moment(), &catalog);
        let titles: Vec<(&str, Vec<String>)> =
            groups.iter().map(|group| (group.title.as_str(), names(&group.rows))).collect();
        assert_eq!(
            titles,
            [
                ("February 2025", vec!["b.pdf".to_owned()]),
                ("October 2024", vec!["a.pdf".to_owned(), "c.pdf".to_owned()])
            ]
        );
    }

    #[test]
    fn senders_group_ignoring_case_with_unknown_last() {
        let catalog = TestCatalog::new()
            .document("a.pdf", |document| {
                document.correspondent("zeta");
            })
            .document("b.pdf", |_| {})
            .document("c.pdf", |document| {
                document.correspondent("Alpha ");
            })
            .document("d.pdf", |document| {
                document.correspondent("ZETA");
            })
            .build();
        let groups = groups(rows(&catalog), GroupKey::Sender, &moment(), &catalog);
        let titles: Vec<(&str, usize)> = groups.iter().map(|group| (group.title.as_str(), group.rows.len())).collect();
        assert_eq!(titles, [("Alpha", 1), ("zeta", 2), ("Unknown sender", 1)]);
    }

    #[test]
    fn tiles_count_subcategories_and_show_the_first_three_kinds_by_path() {
        let catalog = TestCatalog::new()
            .category("c-home", "Home", None)
            .category("c-energy", "Energy", Some("c-home"))
            .category("c-bank", "bank", None)
            .document("d.pdf", |document| {
                document.category("c-energy");
            })
            .document("a.png", |document| {
                document.category("c-home");
            })
            .document("b.docx", |document| {
                document.category("c-home");
            })
            .document("c.txt", |document| {
                document.category("c-home");
            })
            .document("z.pdf", |document| {
                document.category("c-bank");
            })
            .build();
        let tiles = tiles(&catalog);
        assert_eq!(tiles.iter().map(|tile| (tile.name, tile.count)).collect::<Vec<_>>(), [("bank", 1), ("Home", 4)]);
        assert_eq!(tiles[1].kinds, [DocumentKind::Image, DocumentKind::Office, DocumentKind::Other]);
    }

    #[test]
    fn counts_cover_every_sidebar_view() {
        let catalog = TestCatalog::new()
            .settings(|settings| settings.layout.inbox.enabled = true)
            .document("Inbox/a.pdf", |document| {
                document.added("2025-10-02T10:00:00Z");
            })
            .document("x/y/b.pdf", |_| {})
            .document("c.pdf", |_| {})
            .build();
        let review = HashSet::from([
            catalog.documents().keys().next().expect("id").clone(),
            DocumentId::new("gone").expect("id"),
        ]);
        let counts = counts(&catalog, &review, &moment());
        assert_eq!(counts, Counts { all: 3, inbox: 1, needs_review: 1, added_this_month: 1, folders: 3 });
    }
}
