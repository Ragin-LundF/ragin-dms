//! The folded details of a document (Kotlin `DetailsController` reads, `FieldsEditor`, `RetentionEditor`,
//! `LinksEditor`): fields, links, the rule button and retention; and the documents to link to.

use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::field::{FieldType, FieldValues};
use dms_domain::retention::RetentionDates;
use dms_domain::{iso_date, text};

use crate::features::document::dto::{
    CategoryRetention, DocumentDetails, FieldEditor, LinkedDocument, RetentionEditor,
};

/// At most this many documents to link to.
const LINK_MATCHES: usize = 30;

pub fn present_details(catalog: &Catalog, document: &Document) -> DocumentDetails {
    DocumentDetails {
        note: document.note.clone(),
        links: links_of(catalog, document),
        fields: field_editors(catalog, document),
        rule_disabled_reason: document
            .metadata
            .correspondent
            .is_none()
            .then(|| "Give the document a correspondent first".to_owned()),
        retention: retention_of(catalog, document),
    }
}

/// The documents linked with `document`, with their labels, sorted by name ignoring case.
pub fn links_of(catalog: &Catalog, document: &Document) -> Vec<LinkedDocument> {
    let mut links: Vec<LinkedDocument> = catalog
        .links_of(&document.id)
        .into_iter()
        .filter_map(|link| {
            let other = catalog.documents().get(link.other_than(&document.id))?;
            Some(LinkedDocument { label: link.label().map(str::to_owned), ..candidate_of(other) })
        })
        .collect();
    links.sort_by(|first, second| text::compare(&text::lowercase(&first.name), &text::lowercase(&second.name)));
    links
}

/// Documents to link `document` to whose path, title or correspondent contains `query` (ignoring case), newest
/// added first; the document itself is left out.
pub fn link_candidates(catalog: &Catalog, document: &Document, query: &str) -> Vec<LinkedDocument> {
    let wanted = text::lowercase(text::trim(query));
    let mut matches: Vec<&Document> = catalog
        .documents()
        .values()
        .filter(|other| other.id != document.id)
        .filter(|other| wanted.is_empty() || searchable_text(other).contains(&wanted))
        .collect();
    matches.sort_by_key(|other| std::cmp::Reverse(other.added_at));
    matches.into_iter().take(LINK_MATCHES).map(candidate_of).collect()
}

pub fn candidate_of(document: &Document) -> LinkedDocument {
    LinkedDocument {
        document_id: document.id.value().to_owned(),
        name: display_name(document).to_owned(),
        path: document.path.value().to_owned(),
        label: None,
    }
}

/// The title, or the file name without one.
pub fn display_name(document: &Document) -> &str {
    document.metadata.title.as_deref().unwrap_or_else(|| document.path.file_name())
}

pub(crate) fn searchable_text(document: &Document) -> String {
    let parts =
        [Some(document.path.value()), document.metadata.title.as_deref(), document.metadata.correspondent.as_deref()];
    text::lowercase(&parts.into_iter().flatten().collect::<Vec<_>>().join(" "))
}

fn field_editors(catalog: &Catalog, document: &Document) -> Vec<FieldEditor> {
    catalog
        .settings()
        .field_definitions
        .iter()
        .map(|definition| FieldEditor {
            key: definition.key.value().to_owned(),
            label: format!("{} ({})", definition.label, editor_hint(definition.field_type)),
            value: document.fields.get(&definition.key).map(FieldValues::format).unwrap_or_default(),
        })
        .collect()
}

fn editor_hint(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Text => "text",
        FieldType::Number => "number",
        FieldType::Money => "amount and currency",
        FieldType::Date => "YYYY-MM-DD",
        FieldType::Boolean => "yes or no",
    }
}

fn retention_of(catalog: &Catalog, document: &Document) -> RetentionEditor {
    let retention = document.retention.as_ref();
    let status = if retention.is_some_and(|retention| retention.settled_at.is_some()) {
        "Marked as done; never flagged again".to_owned()
    } else if let Some(date) = RetentionDates::due_date(document, catalog) {
        format!("Retention ends on {}", iso_date(date))
    } else {
        "No retention set".to_owned()
    };
    let category = document.category_id.as_ref().and_then(|id| catalog.categories().get(id));
    RetentionEditor {
        status,
        keep_until: retention.and_then(|retention| retention.keep_until).map(iso_date).unwrap_or_default(),
        category: category.map(|category| CategoryRetention {
            id: category.id.value().to_owned(),
            label: format!("Keep documents of \"{}\" for … years", category.name),
            years: category.retention_years.map(|years| years.to_string()).unwrap_or_default(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use dms_domain::retention::Retention;
    use dms_domain::settings::CatalogSettings;
    use dms_domain::taxonomy::Category;

    use super::*;
    use crate::features::document::view::test_catalog::*;

    #[test]
    fn links_show_both_sides_by_name_with_label() {
        let mut first = document(1, "a.pdf");
        first.metadata.title = Some("zeta contract".to_owned());
        let second = document(2, "Home/beta.pdf");
        let third = document(3, "c.pdf");
        let catalog = catalog(
            vec![first.clone(), second.clone(), third.clone()],
            vec![],
            vec![link("d1", "d3", Some("reminder")), link("d2", "d3", None)],
            CatalogSettings::default(),
        );

        let names: Vec<(String, Option<String>)> =
            links_of(&catalog, &third).into_iter().map(|link| (link.name, link.label)).collect();
        assert_eq!(names, [("beta.pdf".to_owned(), None), ("zeta contract".to_owned(), Some("reminder".to_owned()))]);
        assert_eq!(links_of(&catalog, &first)[0].path, "c.pdf", "a link shows on both documents");
    }

    #[test]
    fn link_candidates_match_path_title_or_correspondent_newest_first_without_the_document() {
        let mut power = document(1, "Home/bill.pdf");
        power.metadata.correspondent = Some("City Power".to_owned());
        let mut tax = document(2, "Finance/2025.pdf");
        tax.metadata.title = Some("Power of attorney".to_owned());
        let shown = document(3, "Power/notes.pdf");
        let catalog = catalog(vec![power, tax, shown.clone()], vec![], vec![], CatalogSettings::default());

        let ids = |query: &str| -> Vec<String> {
            link_candidates(&catalog, &shown, query).into_iter().map(|candidate| candidate.document_id).collect()
        };
        assert_eq!(ids(" POWER "), ["d2", "d1"]);
        assert_eq!(ids(""), ["d2", "d1"], "an empty search lists every other document");
        assert_eq!(ids("garden"), Vec::<String>::new());
    }

    #[test]
    fn link_candidates_stop_at_thirty() {
        let documents: Vec<Document> = (1..=31).map(|index| document(index, &format!("doc {index}.pdf"))).collect();
        let shown = documents[0].clone();
        let catalog = catalog(documents, vec![], vec![], CatalogSettings::default());
        let candidates = link_candidates(&catalog, &shown, "doc");
        assert_eq!(candidates.len(), 30);
        assert_eq!(candidates[0].document_id, "d31");
    }

    #[test]
    fn field_editors_label_each_field_with_its_hint_and_formatted_value() {
        let settings = standard_settings();
        let mut bill = document(1, "bill.pdf");
        field_value(&mut bill, "amount", "1.234,5", &settings);
        let catalog = catalog(vec![bill.clone()], vec![], vec![], settings);
        let fields = present_details(&catalog, &bill).fields;
        assert_eq!(
            fields[0],
            FieldEditor {
                key: "amount".to_owned(),
                label: "Amount (amount and currency)".to_owned(),
                value: "1234.5 EUR".to_owned()
            }
        );
        assert_eq!(fields[1].label, "Due date (YYYY-MM-DD)");
        assert_eq!(fields[1].value, "");
    }

    #[test]
    fn rule_needs_a_correspondent() {
        let mut bill = document(1, "bill.pdf");
        let empty = catalog(vec![bill.clone()], vec![], vec![], CatalogSettings::default());
        assert_eq!(
            present_details(&empty, &bill).rule_disabled_reason.as_deref(),
            Some("Give the document a correspondent first")
        );
        bill.metadata.correspondent = Some("City Power".to_owned());
        let named = catalog(vec![bill.clone()], vec![], vec![], CatalogSettings::default());
        assert_eq!(present_details(&named, &bill).rule_disabled_reason, None);
    }

    #[test]
    fn retention_status_prefers_done_then_the_end_date_then_none() {
        let years = Category::new(category_id("c1"), "Tax", None, Some(10)).expect("category");
        let mut by_category = document(1, "a.pdf");
        by_category.category_id = Some(category_id("c1"));
        by_category.metadata.document_date = Some(day("2024-05-01"));
        let mut kept = document(2, "b.pdf");
        kept.retention = Some(Retention { keep_until: Some(day("2030-12-31")), settled_at: None });
        let mut settled = document(3, "c.pdf");
        settled.category_id = Some(category_id("c1"));
        settled.retention =
            Some(Retention { keep_until: None, settled_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single() });
        let plain = document(4, "d.pdf");
        let catalog = catalog(
            vec![by_category.clone(), kept.clone(), settled.clone(), plain.clone()],
            vec![years],
            vec![],
            CatalogSettings::default(),
        );

        let category_retention = present_details(&catalog, &by_category).retention;
        assert_eq!(category_retention.status, "Retention ends on 2034-12-31");
        let category = category_retention.category.expect("category period");
        assert_eq!((category.label.as_str(), category.years.as_str()), ("Keep documents of \"Tax\" for … years", "10"));
        let kept_retention = present_details(&catalog, &kept).retention;
        assert_eq!(
            (kept_retention.status.as_str(), kept_retention.keep_until.as_str()),
            ("Retention ends on 2030-12-31", "2030-12-31")
        );
        assert_eq!(kept_retention.category, None);
        assert_eq!(present_details(&catalog, &settled).retention.status, "Marked as done; never flagged again");
        assert_eq!(present_details(&catalog, &plain).retention.status, "No retention set");
    }
}
