use std::fs;
use std::path::{Path, PathBuf};

use image::RgbImage;
use tempfile::TempDir;

use super::{KeepReason, PageInput, PageLayout, PreparedPage, Rebuilder, RebuiltFile, RegionKind};
use crate::document::{page_of, render_rgb};
use crate::error::PdfError;
use crate::images;
use crate::rebuild::LayoutLine;
use crate::test_support::{TestScan, engine};

const GREY_PRINT: u8 = 165;
const SHADE: u8 = 205;
const LOGO_RED: u8 = 200;
const DARK: u8 = 200;

struct Fixture {
    root: TempDir,
    rebuilder: Rebuilder,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let rebuilder = Rebuilder::new(engine(), root.path().join("work"));
        Fixture { root, rebuilder }
    }

    fn scan(&self) -> PathBuf {
        self.root.path().join("scan.pdf")
    }

    fn work_file(&self, rebuilt: &RebuiltFile) -> PathBuf {
        self.root.path().join("work").join(format!("{}.pdf", rebuilt.id))
    }

    fn prepare(&self) -> PreparedPage {
        self.rebuilder.prepare_page(&self.scan(), 0).expect("prepared")
    }

    fn read(&self, lines: Vec<LayoutLine>) -> PageInput {
        PageInput::Read { prepared: self.prepare(), layout: PageLayout { lines, graphics: Vec::new() } }
    }

    fn rebuild(&self, input: PageInput) -> RebuiltFile {
        self.rebuilder.write(&self.scan(), &"0".repeat(64), &[input]).expect("rebuilt")
    }

    fn text_of(&self, rebuilt: &RebuiltFile) -> String {
        engine().text(&self.work_file(rebuilt)).expect("text")
    }

    fn render_of(&self, rebuilt: &RebuiltFile) -> RgbImage {
        let document = engine().open(&self.work_file(rebuilt)).expect("open");
        render_rgb(&page_of(&document, 0).expect("page"), 300.0).expect("render")
    }
}

fn darkest(image: &RgbImage) -> u8 {
    image.pixels().map(|pixel| pixel.0[2]).min().unwrap_or(255)
}

fn image_filters(file: &Path) -> Vec<String> {
    let document = lopdf::Document::load(file).expect("lopdf");
    document
        .objects
        .values()
        .filter_map(|object| object.as_stream().ok())
        .filter(|stream| stream.dict.get(b"Subtype").and_then(|s| s.as_name()).is_ok_and(|name| name == b"Image"))
        .filter_map(|stream| stream.dict.get(b"Filter").and_then(|f| f.as_name()).ok())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect()
}

#[test]
fn a_page_turned_by_two_degrees_is_straightened_and_its_lines_become_text_with_umlauts() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let lines: Vec<LayoutLine> = [
        "Rechnung Nr. 4711 für Jörg Müller",
        "Beispielstraße 12 in 12345 Musterstadt",
        "Leistung im Zeitraum Januar bis März",
        "Gesamtbetrag 123,45 € inklusive Steuer",
        "Zahlbar bis Ende des Monats ohne Abzug",
        "Vielen Dank für Ihren Auftrag",
    ]
    .iter()
    .zip(0..)
    .map(|(text, i)| scan.dark_text(text, (250, 250 + i * 120)))
    .collect();
    TestScan::write_pdf(&fixture.scan(), &scan.turned(2.0));

    let prepared = fixture.prepare();
    let rebuilt =
        fixture.rebuild(PageInput::Read { prepared: prepared.clone(), layout: PageLayout { lines, graphics: vec![] } });

    assert!((prepared.angle - 2.0).abs() <= 0.1 + 1e-9, "angle {}", prepared.angle);
    let page = &rebuilt.pages[0];
    assert_eq!(page.lines_as_text, 6, "kept {:?}", page.kept_reasons);
    let text = fixture.text_of(&rebuilt);
    assert!(text.contains("für Jörg Müller"), "{text}");
    assert!(text.contains("123,45 €"), "{text}");
}

#[test]
fn a_grey_but_legible_line_becomes_text_through_the_faint_mask() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let dark = scan.dark_text("Kundennummer 99887766 Beispiel", (150, 300));
    let grey = scan.text("Seite 1 von 1 Beispieltext", (150, 450), GREY_PRINT, true, "Seite 1 von 1 Beispieltext");
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(fixture.read(vec![dark, grey]));

    let page = &rebuilt.pages[0];
    assert_eq!(page.lines_as_text, 2, "{:?}", page.kept_reasons);
    assert!(fixture.text_of(&rebuilt).contains("Seite 1 von 1"));
}

#[test]
fn a_reading_shorter_than_the_ink_stays_an_image_as_a_mismatch() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let invented = scan.text("Gesamtbetrag der Rechnung 123,45", (150, 300), 0, true, "Gesamt 5");
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(fixture.read(vec![invented]));

    let page = &rebuilt.pages[0];
    assert_eq!(page.kept_reasons.iter().collect::<Vec<_>>(), vec![(&KeepReason::TextMismatch, &1)]);
    assert!(!fixture.text_of(&rebuilt).contains("Gesamt 5"));
    assert!(page.regions.iter().any(|region| region.kind == RegionKind::Image));
}

#[test]
fn a_faint_line_the_ai_is_not_sure_about_stays_an_image_and_keeps_its_grey() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let faint = scan.text("Kaum lesbarer Vermerk", (150, 300), GREY_PRINT, false, "Kaum lesbarer Vermerk");
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(fixture.read(vec![faint]));

    assert_eq!(rebuilt.pages[0].kept_reasons.iter().collect::<Vec<_>>(), vec![(&KeepReason::NotSure, &1)]);
    assert!(!fixture.text_of(&rebuilt).contains("Vermerk"));
    let original = images::decode(&fixture.prepare().image).expect("png").to_rgb8();
    assert_eq!(darkest(&fixture.render_of(&rebuilt)), darkest(&original), "the line keeps its grey");
}

#[test]
fn a_large_shaded_box_is_stored_as_jpeg_and_keeps_its_grey_value() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    scan.fill((200, 200, 900, 700), [SHADE; 3]);
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(fixture.read(vec![]));

    let filters = image_filters(&fixture.work_file(&rebuilt));
    assert!(filters.iter().any(|filter| filter == "DCTDecode"), "filters {filters:?}");
    let centre = fixture.render_of(&rebuilt).get_pixel(650, 550).0[2];
    assert!(centre.abs_diff(SHADE) <= 1, "grey {centre}");
}

#[test]
fn a_logo_stays_an_image_next_to_the_text() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let line = scan.dark_text("Beispiel GmbH Musterstraße 1", (500, 300));
    scan.fill((150, 200, 250, 150), [LOGO_RED, 30, 30]);
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(fixture.read(vec![line]));

    assert_eq!(rebuilt.pages[0].lines_as_text, 1, "{:?}", rebuilt.pages[0].kept_reasons);
    assert!(fixture.text_of(&rebuilt).contains("Musterstraße"));
    let logo = fixture.render_of(&rebuilt).get_pixel(275, 275).0;
    assert!(logo[0].abs_diff(LOGO_RED) <= 2, "the logo keeps its colour: {logo:?}");
}

#[test]
fn a_page_the_ai_could_not_read_keeps_the_original_page() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    scan.dark_text("Unverändert", (150, 300));
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let rebuilt = fixture.rebuild(PageInput::Failed { page_index: 0 });

    assert_eq!(rebuilt.pages[0].kept_reasons.iter().collect::<Vec<_>>(), vec![(&KeepReason::PageFailed, &1)]);
    assert!(fixture.render_of(&rebuilt).pixels().any(|pixel| pixel.0[2] < DARK));
    let missing = fixture.rebuilder.write(&fixture.scan(), "hash", &[PageInput::Failed { page_index: 3 }]);
    assert!(matches!(missing, Err(PdfError::NotFound)), "{missing:?}");
}

#[test]
fn a_crop_of_a_line_is_a_png_around_its_box() {
    let fixture = Fixture::new();
    let mut scan = TestScan::new();
    let line = scan.dark_text("Zeile zum erneuten Lesen", (150, 300));
    TestScan::write_pdf(&fixture.scan(), &scan.image);

    let crop = fixture.rebuilder.crop(&fixture.prepare(), line.bounds).expect("crop");

    assert_eq!(crop.mime_type, "image/png");
    let decoded = images::decode(&crop).expect("png");
    assert!(
        decoded.width() > 500 && decoded.height() > 40 && decoded.height() < 200,
        "{}x{}",
        decoded.width(),
        decoded.height()
    );
}

#[test]
fn a_rebuild_is_placed_without_replacing_and_renders() {
    let fixture = Fixture::new();
    TestScan::write_pdf(&fixture.scan(), &TestScan::new().image);
    let rebuilt = fixture.rebuild(PageInput::Failed { page_index: 0 });
    let target = fixture.root.path().join("library/scan (real).pdf");

    let placed = fixture.rebuilder.place(&rebuilt, &target).expect("placed");
    let again = fixture.rebuilder.place(&rebuilt, &target);

    assert_eq!(placed.size, rebuilt.size);
    assert!(matches!(again, Err(PdfError::AlreadyExists)));
    assert!(fixture.rebuilder.render_page(&rebuilt, 0, 100).expect("render").is_some());
    assert!(fixture.rebuilder.render_page(&rebuilt, 1, 100).expect("render").is_none());
}

#[test]
fn a_discarded_rebuild_is_gone_and_the_next_start_removes_leftovers() {
    let fixture = Fixture::new();
    TestScan::write_pdf(&fixture.scan(), &TestScan::new().image);
    let first = fixture.rebuild(PageInput::Failed { page_index: 0 });
    let second = fixture.rebuild(PageInput::Failed { page_index: 0 });

    fixture.rebuilder.discard(&first);
    assert!(!fixture.work_file(&first).exists());
    assert!(fixture.work_file(&second).exists());
    let _next_start = Rebuilder::new(engine(), fixture.root.path().join("work"));

    assert!(!fixture.work_file(&second).exists());
    assert!(fs::read_dir(fixture.root.path().join("work")).expect("list").next().is_none());
}

#[test]
fn an_id_that_is_not_a_rebuild_never_leaves_the_work_folder() {
    let fixture = Fixture::new();
    let foreign = RebuiltFile { id: "../scan".into(), pages: vec![], size: 0, source_hash: String::new() };

    let rendered = fixture.rebuilder.render_page(&foreign, 0, 100);

    assert!(matches!(rendered, Err(PdfError::InvalidRequest(_))));
}
