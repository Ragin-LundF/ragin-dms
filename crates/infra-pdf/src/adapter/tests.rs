#![allow(clippy::expect_used)]

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate};
use dms_application::port::ai::{DocumentContent, DocumentContentReader, DocumentPreviewer, PageImage};
use dms_application::port::pdf::{OcrTool, PageChange, PageInput, PdfEditor, PdfMetadata, PdfRebuilder};
use dms_application::port::state::AiSettings;
use dms_application::port::storage::StorageFailure;
use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, RelativePath};
use dms_domain::rebuild::{LayoutBox, PageLayout};
use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;

use super::*;
use crate::test_support::{TestScan, engine, write_text_pdf};
use crate::{CommandRunner, OcrMyPdf};

const SAMPLE_TEXT: &str = "Sample invoice number 4711 for synthetic test services. ";

struct Library {
    root: TempDir,
    folder: LibraryFolder,
}

impl Library {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let location = FolderLocation::new(root.path().to_string_lossy()).expect("location");
        let folder =
            LibraryFolder::new(LibraryFolderId::new("primary").expect("id"), "Primary", location, FolderRole::Primary)
                .expect("folder");
        Self { root, folder }
    }

    fn file(&self, name: &str) -> std::path::PathBuf {
        self.root.path().join(name)
    }

    /// A library folder whose location does not exist, like an offline share.
    fn unreachable(&self) -> LibraryFolder {
        LibraryFolder {
            location: FolderLocation::new(self.root.path().join("offline").to_string_lossy()).expect("location"),
            ..self.folder.clone()
        }
    }
}

fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("path")
}

fn document(name: &str) -> Document {
    Document::new(
        DocumentId::new(name).expect("id"),
        path(name),
        ContentHash::new("a".repeat(64)).expect("hash"),
        1,
        DateTime::from_timestamp(1_767_225_600, 0).expect("time"),
    )
    .expect("document")
}

fn reader_with(settings: AiSettings) -> FileContentReader {
    FileContentReader::new(engine(), Arc::new(move || settings.clone()))
}

fn decoded(image: &PageImage) -> DynamicImage {
    image::load_from_memory(&image.bytes).expect("a picture")
}

fn write_png(file: &Path, width: u32, height: u32) {
    DynamicImage::ImageRgba8(RgbaImage::new(width, height)).save_with_format(file, ImageFormat::Png).expect("png");
}

/// A minimal Office file of `parts` (name, XML).
fn write_zip(file: &Path, parts: &[(&str, &str)]) {
    let mut zip = zip::ZipWriter::new(fs::File::create(file).expect("create"));
    for (name, xml) in parts {
        zip.start_file(*name, SimpleFileOptions::default()).expect("entry");
        zip.write_all(xml.as_bytes()).expect("write");
    }
    zip.finish().expect("finish");
}

fn write_docx(file: &Path, paragraphs: &[&str]) {
    let body =
        paragraphs.iter().fold(String::new(), |body, text| body + &format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"));
    let xml = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );
    write_zip(file, &[("word/document.xml", &xml)]);
}

// --- document reader ---

#[test]
fn a_pdf_with_a_text_layer_is_read_as_text() {
    let library = Library::new();
    write_text_pdf(&library.file("a.pdf"), &[Some(&SAMPLE_TEXT.repeat(4))]);

    let content = reader_with(AiSettings::default()).read(&library.folder, &document("a.pdf"));

    let DocumentContent::Text(text) = content else { panic!("no text: {content:?}") };
    assert!(text.contains("Sample invoice number 4711"));
}

#[test]
fn the_text_is_cut_after_the_configured_input_budget() {
    let library = Library::new();
    write_text_pdf(&library.file("long.pdf"), &[Some(&SAMPLE_TEXT.repeat(30)), Some(&SAMPLE_TEXT.repeat(30))]);

    let content = reader_with(AiSettings { input_characters: 1_000, ..AiSettings::default() })
        .read(&library.folder, &document("long.pdf"));

    let DocumentContent::Text(text) = content else { panic!("no text: {content:?}") };
    assert_eq!(text.chars().count(), 1_000);
}

#[test]
fn the_text_for_search_is_the_whole_text_layer_and_nothing_for_a_scan_or_an_image() {
    let library = Library::new();
    write_text_pdf(&library.file("a.pdf"), &[Some(&SAMPLE_TEXT.repeat(4))]);
    write_text_pdf(&library.file("scan.pdf"), &[None]);
    write_png(&library.file("photo.png"), 10, 10);
    let reader = reader_with(AiSettings::default());

    let text = reader.text(&library.folder, &document("a.pdf"));

    assert!(text.expect("text").contains("Sample invoice number 4711"));
    assert_eq!(reader.text(&library.folder, &document("scan.pdf")), None);
    assert_eq!(reader.text(&library.folder, &document("photo.png")), None);
    assert_eq!(reader.text(&library.folder, &document("gone.pdf")), None);
}

#[test]
fn a_pdf_without_text_or_with_vision_preferred_is_sent_as_page_images() {
    let library = Library::new();
    write_text_pdf(&library.file("scan.pdf"), &[None]);
    write_text_pdf(&library.file("text.pdf"), &[Some(&SAMPLE_TEXT.repeat(4))]);

    let scan = reader_with(AiSettings::default()).read(&library.folder, &document("scan.pdf"));
    let preferred = reader_with(AiSettings { prefer_vision_for_pdfs: true, ..AiSettings::default() })
        .read(&library.folder, &document("text.pdf"));

    let DocumentContent::PageImages(pages) = scan else { panic!("no images: {scan:?}") };
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].mime_type, "image/jpeg");
    assert!(matches!(preferred, DocumentContent::PageImages(_)), "{preferred:?}");
}

#[test]
fn an_image_is_downscaled_for_the_model_whatever_its_extension_says() {
    let library = Library::new();
    write_png(&library.file("photo.jpg"), 4000, 2000);

    let content = reader_with(AiSettings::default()).read(&library.folder, &document("photo.jpg"));

    let DocumentContent::PageImages(pages) = content else { panic!("no images: {content:?}") };
    let image = decoded(&pages[0]);
    assert_eq!((image.width(), image.height()), (1600, 800));
}

#[test]
fn a_word_file_is_read_as_text_with_its_entities() {
    let library = Library::new();
    write_docx(&library.file("contract.docx"), &["Synthetic contract text", "Fees &amp; costs &#8364;"]);

    let content = reader_with(AiSettings::default()).read(&library.folder, &document("contract.docx"));

    assert!(
        matches!(&content, DocumentContent::Text(text) if text == "Synthetic contract text\nFees & costs €"),
        "{content:?}"
    );
}

#[test]
fn a_presentation_is_read_slide_by_slide_in_order() {
    let library = Library::new();
    let slide = |text: &str| {
        format!(
            r#"<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:p><a:r><a:t>{text}</a:t></a:r></a:p></p:sld>"#
        )
    };
    let (second, tenth, first) = (slide("Second"), slide("Tenth"), slide("First"));
    write_zip(
        &library.file("talk.pptx"),
        &[("ppt/slides/slide2.xml", &second), ("ppt/slides/slide10.xml", &tenth), ("ppt/slides/slide1.xml", &first)],
    );

    let text = reader_with(AiSettings::default()).text(&library.folder, &document("talk.pptx"));

    assert_eq!(text.as_deref(), Some("First\nSecond\nTenth"));
}

#[test]
fn a_spreadsheet_is_read_with_its_sheet_names_and_rows() {
    let library = Library::new();
    write_zip(
        &library.file("budget.xlsx"),
        &[
            (
                "[Content_Types].xml",
                r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
            ),
            (
                "xl/workbook.xml",
                r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Budget" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
            ),
            (
                "xl/worksheets/sheet1.xml",
                r#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Rent</t></is></c><c r="B1"><v>950</v></c></row></sheetData></worksheet>"#,
            ),
        ],
    );

    let text = reader_with(AiSettings::default()).text(&library.folder, &document("budget.xlsx"));

    assert_eq!(text.as_deref(), Some("Budget\nRent\t950"));
}

#[test]
fn a_missing_file_a_broken_office_file_an_empty_one_or_another_kind_is_not_readable() {
    let library = Library::new();
    fs::write(library.file("song.mp3"), "x").expect("write");
    fs::write(library.file("broken.docx"), "not a zip").expect("write");
    write_docx(&library.file("empty.docx"), &[]);
    let reader = reader_with(AiSettings::default());

    for name in ["gone.pdf", "song.mp3", "broken.docx", "empty.docx", "broken.pdf"] {
        let content = reader.read(&library.folder, &document(name));
        assert!(matches!(content, DocumentContent::NotReadable(_)), "{name}: {content:?}");
    }
    let empty = reader.read(&library.folder, &document("empty.docx"));
    assert!(matches!(&empty, DocumentContent::NotReadable(reason) if reason == "The file contains no text"));
}

// --- previewer ---

#[test]
fn a_pdf_shows_its_first_page_scaled_down_and_an_image_is_scaled_to_the_thumbnail_size() {
    let library = Library::new();
    write_text_pdf(&library.file("a.pdf"), &[None]);
    let mut photo = Vec::new();
    DynamicImage::ImageRgb8(RgbImage::new(800, 400))
        .write_to(&mut Cursor::new(&mut photo), ImageFormat::Png)
        .expect("png");
    fs::write(library.file("photo.png"), photo).expect("write");
    let previewer = FilePreviewer::new(engine());

    let page = previewer.thumbnail(&library.folder, &path("a.pdf"), 120).expect("thumbnail");
    let photo = previewer.thumbnail(&library.folder, &path("photo.png"), 200).expect("thumbnail");

    let page = decoded(&page);
    assert_eq!(page.width().max(page.height()), 120);
    let photo = decoded(&photo);
    assert_eq!((photo.width(), photo.height()), (200, 100));
}

#[test]
fn office_files_missing_files_and_broken_files_have_no_thumbnail() {
    let library = Library::new();
    write_docx(&library.file("letter.docx"), &["Hello"]);
    fs::write(library.file("broken.pdf"), "not a pdf").expect("write");
    let previewer = FilePreviewer::new(engine());

    for name in ["letter.docx", "missing.pdf", "broken.pdf"] {
        assert!(previewer.thumbnail(&library.folder, &path(name), 100).is_none(), "{name}");
    }
}

// --- locations ---

#[test]
fn a_file_named_decomposed_on_disk_is_found_by_its_composed_name_and_a_new_one_goes_into_that_folder() {
    let library = Library::new();
    fs::create_dir(library.file("Mu\u{308}ller")).expect("folder");
    fs::write(library.file("Mu\u{308}ller/Rechnung.pdf"), "x").expect("write");

    let existing = LibraryLocations.existing(&library.folder, &path("M\u{fc}ller/Rechnung.pdf"));
    let target = LibraryLocations.target(&library.folder, &path("M\u{fc}ller/Neu/Brief.pdf")).expect("target");

    // Some file systems find the composed name directly, others only through the NFC comparison.
    assert_eq!(fs::read_to_string(existing.expect("found")).expect("read"), "x");
    let folder = target.parent().and_then(Path::parent).expect("folder");
    assert!(folder.is_dir(), "{target:?} goes into the existing folder");
    assert!(target.ends_with("Neu/Brief.pdf"));
    assert_eq!(LibraryLocations.existing(&library.folder, &path("M\u{fc}ller/Fehlt.pdf")), None);
    assert_eq!(LibraryLocations.target(&library.unreachable(), &path("a.pdf")), None);
}

// --- editor ---

#[test]
fn the_editor_rewrites_merges_and_reports_the_written_files_facts() {
    let library = Library::new();
    write_text_pdf(&library.file("a.pdf"), &[Some("one"), Some("two"), Some("three")]);
    let editor = PdfiumEditor::new(engine());
    let pages = [PageChange { page: 2, rotation: 90 }, PageChange { page: 0, rotation: 0 }];

    let rewritten = editor.rewrite(&library.folder, &path("a.pdf"), &pages, &path("out/b.pdf")).expect("rewritten");
    let merged = editor.merge(&library.folder, &[path("a.pdf"), path("out/b.pdf")], &path("c.pdf")).expect("merged");

    assert_eq!(rewritten.size, fs::metadata(library.file("out/b.pdf")).expect("written").len());
    assert!(rewritten.content_hash.is_some());
    assert_eq!(editor.page_count(&library.folder, &path("out/b.pdf")), Ok(2));
    assert_eq!(editor.page_count(&library.folder, &path("c.pdf")), Ok(5));
    assert_ne!(merged.content_hash, rewritten.content_hash);
    assert!(editor.render_page(&library.folder, &path("c.pdf"), 0, 100).is_some());
    assert!(editor.render_page(&library.folder, &path("c.pdf"), 9, 100).is_none());
}

#[test]
fn a_missing_source_is_not_found_an_existing_target_refused_and_an_offline_folder_unreachable() {
    let library = Library::new();
    write_text_pdf(&library.file("a.pdf"), &[Some("one")]);
    let editor = PdfiumEditor::new(engine());
    let all = [PageChange { page: 0, rotation: 0 }];

    let missing = editor.rewrite(&library.folder, &path("gone.pdf"), &all, &path("b.pdf"));
    let taken = editor.rewrite(&library.folder, &path("a.pdf"), &all, &path("A.PDF"));
    let merge_missing = editor.merge(&library.folder, &[path("a.pdf"), path("gone.pdf")], &path("c.pdf"));
    let offline = editor.page_count(&library.unreachable(), &path("a.pdf"));

    assert_eq!(missing.map_err(|error| error.reason), Err(StorageFailure::NotFound));
    assert_eq!(taken.map_err(|error| error.reason), Err(StorageFailure::AlreadyExists));
    assert_eq!(merge_missing.map_err(|error| error.reason), Err(StorageFailure::NotFound));
    assert_eq!(offline.map_err(|error| error.reason), Err(StorageFailure::NotFound));
}

#[test]
fn a_file_that_is_no_pdf_is_an_unsupported_format() {
    let library = Library::new();
    fs::write(library.file("fake.pdf"), "not a pdf").expect("write");

    let result = PdfiumEditor::new(engine()).has_text_layer(&library.folder, &path("fake.pdf"));

    assert_eq!(result.map_err(|error| error.reason), Err(StorageFailure::UnsupportedFormat));
}

#[test]
fn metadata_and_text_layers_round_trip_through_the_ports_types() {
    let library = Library::new();
    write_text_pdf(&library.file("scan.pdf"), &[None]);
    let editor = PdfiumEditor::new(engine());
    let metadata = PdfMetadata {
        title: Some("Sample bill".to_owned()),
        author: Some("Sample Energy".to_owned()),
        subject: Some("Invoices / Energy".to_owned()),
        keywords: vec!["Tax".to_owned(), "Power".to_owned()],
        date: NaiveDate::from_ymd_opt(2026, 1, 15),
    };
    let text = "Synthetic searchable text on the scanned page".to_owned();

    editor.write_metadata(&library.folder, &path("scan.pdf"), &metadata, &path("meta.pdf")).expect("metadata");
    editor.add_text_layer(&library.folder, &path("scan.pdf"), &[text], &path("text.pdf")).expect("text layer");

    assert_eq!(editor.read_metadata(&library.folder, &path("meta.pdf")), Ok(metadata));
    assert_eq!(editor.has_text_layer(&library.folder, &path("scan.pdf")), Ok(false));
    assert_eq!(editor.has_text_layer(&library.folder, &path("text.pdf")), Ok(true));
}

// --- OCR ---

struct Copying(Mutex<usize>);

impl CommandRunner for Copying {
    fn run(&self, command: &[std::ffi::OsString]) -> Option<i32> {
        *self.0.lock().expect("lock") += 1;
        if command.len() > 2 {
            fs::copy(&command[command.len() - 2], &command[command.len() - 1]).ok()?;
        }
        Some(0)
    }
}

#[test]
fn ocr_writes_the_searchable_copy_to_the_target_and_needs_an_existing_source() {
    let library = Library::new();
    fs::write(library.file("scan.pdf"), "%PDF scan").expect("write");
    let tool = OcrMyPdfTool::new(OcrMyPdf::new(Box::new(Copying(Mutex::new(0))), "ocrmypdf"));

    let facts = tool.ocr(&library.folder, &path("scan.pdf"), &path("ocr.pdf")).expect("ocr");
    let missing = tool.ocr(&library.folder, &path("gone.pdf"), &path("other.pdf"));
    let offline = tool.ocr(&library.unreachable(), &path("scan.pdf"), &path("other.pdf"));

    assert!(tool.available());
    assert_eq!(facts.size, 9);
    assert_eq!(fs::read_to_string(library.file("ocr.pdf")).expect("read"), "%PDF scan");
    assert_eq!(missing.map_err(|error| error.reason), Err(StorageFailure::NotFound));
    assert_eq!(offline.map_err(|error| error.reason), Err(StorageFailure::NotFound));
}

// --- rebuild ---

#[test]
fn a_scan_is_prepared_rebuilt_shown_placed_and_discarded_through_the_port() {
    let library = Library::new();
    let work = tempfile::tempdir().expect("work dir");
    let mut scan = TestScan::new();
    let line = scan.dark_text("Synthetic rebuild sample", (200, 300));
    TestScan::write_pdf(&library.file("scan.pdf"), &scan.image);
    let rebuilder = PdfiumRebuilder::new(engine(), work.path().to_path_buf());
    let hash = ContentHash::new("b".repeat(64)).expect("hash");

    let prepared = rebuilder.prepare_page(&library.folder, &path("scan.pdf"), 0).expect("prepared");
    let crop = rebuilder.crop(&prepared, &LayoutBox { left: 100.0, top: 200.0, right: 400.0, bottom: 300.0 });
    let layout = PageLayout { lines: vec![domain_line(&line)], graphics: Vec::new() };
    let pages = [PageInput::Read { prepared, layout }, PageInput::Failed { page_index: 0 }];
    let rebuilt = rebuilder.write(&library.folder, &path("scan.pdf"), &hash, &pages).expect("rebuilt");

    assert_ne!(crop.bytes, Vec::<u8>::new());
    assert_eq!(rebuilt.pages.len(), 2);
    assert_eq!(rebuilt.pages[0].lines_total, 1);
    assert_eq!(rebuilt.source_hash, hash);
    assert!(rebuilder.render_page(&rebuilt, 1, 100).is_some());
    let placed = rebuilder.place(&rebuilt, &library.folder, &path("real.pdf")).expect("placed");
    assert_eq!(placed.size, rebuilt.size);
    assert_eq!(
        rebuilder.place(&rebuilt, &library.folder, &path("real.pdf")).map_err(|error| error.reason),
        Err(StorageFailure::AlreadyExists)
    );
    rebuilder.discard(&rebuilt);
    assert!(rebuilder.render_page(&rebuilt, 0, 100).is_none());
    assert_eq!(
        rebuilder.prepare_page(&library.folder, &path("scan.pdf"), 5).map(|_| ()).map_err(|error| error.reason),
        Err(StorageFailure::NotFound)
    );
}

#[test]
fn a_crop_of_a_picture_that_is_no_image_is_empty() {
    let work = tempfile::tempdir().expect("work dir");
    let rebuilder = PdfiumRebuilder::new(engine(), work.path().to_path_buf());
    let prepared = dms_application::port::pdf::PreparedPage {
        page_index: 0,
        image: PageImage { bytes: vec![1, 2, 3], mime_type: "image/png".to_owned() },
        angle: 0.0,
    };

    let crop = rebuilder.crop(&prepared, &LayoutBox { left: 0.0, top: 0.0, right: 10.0, bottom: 10.0 });

    assert_eq!(crop.bytes, Vec::<u8>::new());
}

fn domain_line(line: &crate::LayoutLine) -> dms_domain::rebuild::LayoutLine {
    dms_domain::rebuild::LayoutLine {
        bounds: LayoutBox {
            left: line.bounds.left,
            top: line.bounds.top,
            right: line.bounds.right,
            bottom: line.bounds.bottom,
        },
        text: line.text.clone(),
        bold: line.bold,
        family: dms_domain::rebuild::FontFamily::Sans,
        legible: line.legible,
        kept: None,
    }
}
