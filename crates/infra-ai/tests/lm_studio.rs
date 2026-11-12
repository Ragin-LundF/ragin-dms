//! Sends synthetic data to the local model the user runs: an invoice page to read, a 14-page stack of six invented
//! documents to split, and the plain-word searches of the prototype. Opt-in only:
//! `DMS_AI_INTEGRATION=true cargo test -p dms-infra-ai --test lm_studio`, optionally with `DMS_AI_BASE_URL` and
//! `DMS_AI_MODEL`. Without the variable the test passes without doing anything.
#![allow(clippy::expect_used)]

use std::io::Cursor;
use std::sync::Arc;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use chrono::NaiveDate;
use dms_application::port::ai::{LayoutOutcome, LineOutcome, PageImage, PageLayoutReader};
use dms_application::port::assistant::{
    QueryAnswer, QueryRequest, QueryTranslator, SplitAnswer, SplitPage, SplitProposer, SplitRequest, TranslatedQuery,
};
use dms_application::port::state::AiSettings;
use dms_domain::bill::BillStatus;
use dms_domain::field::FieldPresets;
use dms_domain::query::FieldComparison;
use dms_domain::rebuild::LayoutBox;
use dms_domain::tax::TaxArea;
use dms_domain::taxonomy::{Category, CategoryId};
use dms_infra_ai::AiAdapters;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

const LINES: [&str; 12] = [
    "Beispiel Handwerk GmbH",
    "Musterstraße 12, 12345 Beispielstadt",
    "Rechnung Nr. 2026-0815",
    "Rechnungsdatum: 14.03.2026",
    "Herrn Jürgen Beispiel",
    "Gartenweg 7, 54321 Übungsdorf",
    "Pos. 1 Wartung der Heizungsanlage 240,00 €",
    "Pos. 2 Prüfung der Abgaswerte 85,50 €",
    "Zwischensumme 325,50 €",
    "Umsatzsteuer 19 % 61,85 €",
    "Gesamtbetrag 387,35 €",
    "Zahlbar innerhalb von 14 Tagen ohne Abzüge.",
];

/// The adapters on the model of `DMS_AI_BASE_URL` / `DMS_AI_MODEL`, or `None` unless `DMS_AI_INTEGRATION=true`.
fn integration() -> Option<AiAdapters> {
    if std::env::var("DMS_AI_INTEGRATION").as_deref() != Ok("true") {
        return None;
    }
    let settings = AiSettings {
        base_url: std::env::var("DMS_AI_BASE_URL").unwrap_or_else(|_| AiSettings::DEFAULT_BASE_URL.to_owned()),
        model: std::env::var("DMS_AI_MODEL").ok(),
        timeout_seconds: 900,
        ..AiSettings::default()
    };
    Some(AiAdapters::new(Arc::new(move || settings.clone())))
}

#[test]
fn the_local_model_reads_most_lines_of_a_synthetic_invoice() {
    let Some(adapters) = integration() else { return };
    let reader = adapters.layout_reader;
    let page = invoice();

    let LayoutOutcome::Read(layout) = reader.read_layout(&png(&page)) else { panic!("the page was not read") };

    let lines: Vec<String> = layout
        .lines
        .iter()
        .map(|line| {
            // A NUL is a character lost under the JSON grammar; the rebuild reads such a line again from its crop.
            let text = if line.text.contains('\0') {
                match reader.read_line(&png(&crop_of(&page, line.bounds))) {
                    LineOutcome::Read(text) => text,
                    LineOutcome::Failure { .. } => line.text.clone(),
                }
            } else {
                line.text.clone()
            };
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect();
    let found = LINES.iter().filter(|line| lines.iter().any(|read| read == *line)).count();
    assert!(found * 10 >= LINES.len() * 8, "found {found} of {}: {lines:?}", LINES.len());
}

/// An A4 page at 200 dpi with the synthetic invoice lines in Liberation Sans.
fn invoice() -> RgbImage {
    let font =
        FontRef::try_from_slice(include_bytes!("../../infra-pdf/fonts/LiberationSans-Regular.ttf")).expect("font");
    let scaled = font.as_scaled(PxScale::from(44.0));
    let mut page = RgbImage::from_pixel(1654, 2339, Rgb([255, 255, 255]));
    for (index, text) in LINES.iter().enumerate() {
        let mut x = 160.0;
        #[allow(clippy::cast_precision_loss)] // a dozen lines
        let baseline = 240.0 + index as f32 * 90.0;
        for character in text.chars() {
            let glyph = scaled.scaled_glyph(character);
            let advance = scaled.h_advance(glyph.id);
            if let Some(outline) = font.outline_glyph(ab_glyph::Glyph { position: point(x, baseline), ..glyph }) {
                let bounds = outline.px_bounds();
                outline.draw(|gx, gy, coverage| {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // inside the page
                    let (px, py) =
                        ((bounds.min.x as i64 + i64::from(gx)) as u32, (bounds.min.y as i64 + i64::from(gy)) as u32);
                    if px < page.width() && py < page.height() {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // coverage is 0..1
                        let ink = 255 - (coverage.clamp(0.0, 1.0) * 255.0) as u8;
                        let pixel = page.get_pixel_mut(px, py);
                        pixel.0 = pixel.0.map(|channel| channel.min(ink));
                    }
                });
            }
            x += advance;
        }
    }
    page
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // boxes are within 0..1000 of the page
fn crop_of(page: &RgbImage, bounds: LayoutBox) -> RgbImage {
    let pixels = |value: f64, size: u32| (value * f64::from(size) / LayoutBox::SCALE) as u32;
    let x = pixels(bounds.left, page.width()).min(page.width() - 1);
    let y = pixels(bounds.top, page.height()).min(page.height() - 1);
    let right = pixels(bounds.right, page.width()).clamp(x + 1, page.width());
    let bottom = pixels(bounds.bottom, page.height()).clamp(y + 1, page.height());
    image::imageops::crop_imm(page, x, y, right - x, bottom - y).to_image()
}

fn png(image: &RgbImage) -> PageImage {
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(image.clone()).write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).expect("png");
    PageImage { bytes, mime_type: "image/png".to_owned() }
}

/// The page texts of six invented documents on 14 pages, starting on pages 1, 3, 6, 8, 10 and 14; page 7 is blank.
const STACK: [&str; 14] = [
    "Stadtwerke Beispielstadt GmbH · Hafenstraße 3 · 12345 Beispielstadt\nHerrn Jürgen Beispiel, Gartenweg 7\nStromrechnung Nr. 2026-1103 vom 02.03.2026\nAbrechnungszeitraum 01.02.2026 bis 28.02.2026\nSeite 1 von 2",
    "Stadtwerke Beispielstadt · Rechnung 2026-1103 · Seite 2 von 2\nVerbrauch 245 kWh, Arbeitspreis 0,32 €/kWh\nGesamtbetrag 94,00 €, fällig am 16.03.2026",
    "Finanzamt Beispielstadt · Steuernummer 12/345/67890\nHerrn Jürgen Beispiel\nBescheid für 2025 über Einkommensteuer vom 14.04.2026\nSeite 1 von 3",
    "Finanzamt Beispielstadt · Bescheid 2025 · Seite 2 von 3\nBerechnung der Einkommensteuer, zu versteuerndes Einkommen 41.230 €",
    "Finanzamt Beispielstadt · Bescheid 2025 · Seite 3 von 3\nErläuterungen und Rechtsbehelfsbelehrung: Einspruch innerhalb eines Monats",
    "Löwen-Apotheke · Marktplatz 1 · Beispielstadt\nQuittung Nr. 88123 vom 20.04.2026\nIbuprofen 400 mg 5,49 €, Nasenspray 3,95 €\nSumme 9,44 € bar bezahlt",
    "",
    "Zahnarztpraxis Dr. Hansen · Lindenallee 5 · Beispielstadt\nPrivatrechnung Nr. R-7781 vom 05.05.2026\nPatient: Jürgen Beispiel\nSeite 1 von 2",
    "Zahnarztpraxis Dr. Hansen · Rechnung R-7781 · Seite 2 von 2\nGOZ 2060 Füllung 112,50 €, Gesamt 212,75 €, zahlbar bis 05.06.2026",
    "Küstenversicherung AG · Policenservice\nVersicherungsschein Kfz-Versicherung Nr. VS 99-102-334\nVersicherungsbeginn 01.01.2027, Ausstellungsdatum 28.10.2026\nSeite 1 von 4",
    "Küstenversicherung AG · Versicherungsschein VS 99-102-334 · Seite 2 von 4\nFahrzeugdaten, Typklassen, Regionalklasse",
    "Küstenversicherung AG · Versicherungsschein VS 99-102-334 · Seite 3 von 4\nBeitragsübersicht: Haftpflicht 312,40 €, Teilkasko 174,80 €",
    "Küstenversicherung AG · Seite 4 von 4\nAllgemeine Bedingungen für die Kraftfahrtversicherung (AKB 2027)",
    "Deutsche Gesellschaft zur Rettung Schiffbrüchiger · Spendenbescheinigung\nZuwendungsbestätigung über Geldzuwendungen vom 12.06.2026\nBetrag 150,00 €, Jürgen Beispiel",
];

fn category(id: &str, name: &str) -> Category {
    Category::new(CategoryId::new(id).expect("id"), name, None, None).expect("category")
}

#[test]
fn the_local_model_finds_the_six_documents_of_a_fourteen_page_stack() {
    let Some(adapters) = integration() else { return };
    let request = SplitRequest {
        pages: STACK.iter().map(|text| SplitPage { text: (*text).to_owned(), blank: text.is_empty() }).collect(),
        categories: vec![category("bills", "Bills"), category("tax", "Tax"), category("insurance", "Insurance")],
    };

    let SplitAnswer::Proposed { parts, .. } = adapters.split_proposer.propose(&request) else { panic!("no proposal") };

    let starts: Vec<usize> = parts.iter().map(|part| part.first_page).collect();
    assert_eq!(starts, [0, 2, 5, 7, 9, 13], "{parts:?}");
    assert_eq!(parts.last().map(|part| part.last_page), Some(13), "{parts:?}");
}

#[test]
fn the_local_model_turns_the_prototype_searches_into_filters_and_names_an_unknown_sender_as_written() {
    let Some(adapters) = integration() else { return };
    let ask = |sentence: &str| {
        let request = QueryRequest {
            sentence: sentence.to_owned(),
            today: NaiveDate::from_ymd_opt(2026, 11, 11).expect("date"),
            categories: vec![
                category("bill", "Bill"),
                category("insurance", "Insurance"),
                category("letter", "Letter"),
            ],
            tags: Vec::new(),
            correspondents: vec!["Stadtwerke Lundhafen".to_owned(), "Küstenversicherung".to_owned()],
            fields: vec![FieldPresets::amount(), FieldPresets::due_date()],
            tax_areas: TaxArea::ALL.to_vec(),
        };
        match adapters.query_translator.translate(&request) {
            QueryAnswer::Translated { query, .. } => query,
            QueryAnswer::Failed { reason, detail } => panic!("{sentence}: {reason:?} {detail:?}"),
        }
    };

    let bills = ask("Stadtwerke bills 2025 over 50 euros");
    let unpaid = ask("unpaid bills");
    let insurance = ask("insurance letters since 2024");
    let unknown = ask("letters from Nordlicht Bausparkasse");

    assert_eq!(bills.category.as_deref(), Some("Bill"), "{bills:?}");
    assert_eq!(bills.correspondent.as_deref(), Some("Stadtwerke Lundhafen"), "{bills:?}");
    assert_eq!(
        (bills.date_from, bills.date_to),
        (NaiveDate::from_ymd_opt(2025, 1, 1), NaiveDate::from_ymd_opt(2025, 12, 31))
    );
    assert!(
        bills.fields.iter().any(|field| field.key == "amount" && field.comparison == FieldComparison::GreaterThan),
        "{bills:?}"
    );
    assert_eq!((unpaid.category.as_deref(), unpaid.payment), (Some("Bill"), Some(BillStatus::Open)), "{unpaid:?}");
    assert_eq!(insurance.category.as_deref(), Some("Insurance"), "{insurance:?}");
    assert_eq!((insurance.date_from, insurance.date_to), (NaiveDate::from_ymd_opt(2024, 1, 1), None), "{insurance:?}");
    let sender = unknown.correspondent.clone().unwrap_or_default();
    assert!(sender.contains("Nordlicht"), "an unknown sender is written as asked, not swapped: {unknown:?}");
    assert_ne!(unknown, TranslatedQuery::default());
}
