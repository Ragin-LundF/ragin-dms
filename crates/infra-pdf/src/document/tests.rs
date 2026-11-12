use std::fs;
use std::path::Path;

use crate::document::{DocumentContent, PageChange, PdfDate, PdfMetadata};
use crate::error::PdfError;
use crate::images;
use crate::test_support::{engine, write_text_pdf};

fn change(page: usize, rotation: i32) -> PageChange {
    PageChange { page, rotation }
}

fn names_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(folder).expect("list").flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

#[test]
fn pages_are_reordered_turned_and_left_out_in_a_new_file_and_the_source_stays_as_it_was() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("first page"), Some("second page"), Some("third page")]);
    let before = fs::read(&scan).expect("read");
    let target = root.path().join("out/edited.pdf");

    let written = engine().rewrite(&scan, &[change(2, 90), change(0, 0)], &target).expect("rewritten");

    assert_eq!(engine().page_texts(&target).expect("texts"), vec!["third page", "first page"]);
    assert_eq!(written.size, fs::metadata(&target).expect("metadata").len());
    assert_eq!(fs::read(&scan).expect("read"), before);
    assert_eq!(engine().page_count(&scan).expect("count"), 3);
    let turned = images::decode(&engine().render_page(&target, 0, 200).expect("render").expect("page")).expect("png");
    let upright = images::decode(&engine().render_page(&target, 1, 200).expect("render").expect("page")).expect("png");
    assert!(turned.width() > turned.height(), "the turned page is landscape");
    assert!(upright.width() < upright.height());
}

#[test]
fn turning_adds_to_the_rotation_a_page_already_has() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("page")]);
    let once = root.path().join("once.pdf");
    let twice = root.path().join("twice.pdf");
    engine().rewrite(&scan, &[change(0, 90)], &once).expect("turned once");

    engine().rewrite(&once, &[change(0, -180)], &twice).expect("turned back");

    let page = images::decode(&engine().render_page(&twice, 0, 200).expect("render").expect("page")).expect("png");
    assert!(page.width() > page.height(), "90 - 180 = 270 degrees is landscape");
}

#[test]
fn a_rewrite_refuses_no_pages_missing_pages_and_crooked_turns() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("page")]);
    let target = root.path().join("out.pdf");

    assert!(matches!(engine().rewrite(&scan, &[], &target), Err(PdfError::InvalidRequest(_))));
    assert!(matches!(engine().rewrite(&scan, &[change(1, 0)], &target), Err(PdfError::InvalidRequest(_))));
    assert!(matches!(engine().rewrite(&scan, &[change(0, 45)], &target), Err(PdfError::InvalidRequest(_))));
    assert!(matches!(
        engine().rewrite(&root.path().join("gone.pdf"), &[change(0, 0)], &target),
        Err(PdfError::NotFound)
    ));
    assert_eq!(names_in(root.path()), vec!["scan.pdf"]);
}

#[test]
fn merging_keeps_the_order_of_the_sources() {
    let root = tempfile::tempdir().expect("temp dir");
    let (a, b) = (root.path().join("a.pdf"), root.path().join("b.pdf"));
    write_text_pdf(&a, &[Some("alpha one"), Some("alpha two")]);
    write_text_pdf(&b, &[Some("beta one")]);
    let merged = root.path().join("merged.pdf");

    engine().merge(&[&b, &a], &merged).expect("merged");

    assert_eq!(engine().page_texts(&merged).expect("texts"), vec!["beta one", "alpha one", "alpha two"]);
}

#[test]
fn a_write_never_replaces_an_existing_file_and_leaves_the_original_untouched() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("page")]);
    let taken = root.path().join("taken.pdf");
    fs::write(&taken, "someone else's file").expect("write");

    let written = engine().rewrite(&scan, &[change(0, 0)], &taken);
    let broken = engine().rewrite(&taken, &[change(0, 0)], &root.path().join("new.pdf"));

    assert!(matches!(written, Err(PdfError::AlreadyExists)), "{written:?}");
    assert_eq!(fs::read_to_string(&taken).expect("read"), "someone else's file");
    assert!(matches!(broken, Err(PdfError::UnsupportedFormat(_))), "{broken:?}");
    assert_eq!(names_in(root.path()), vec!["scan.pdf", "taken.pdf"]);
}

#[test]
fn an_invisible_text_layer_makes_a_scan_searchable() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[None, None]);
    let searchable = root.path().join("searchable.pdf");
    let texts = ["Sample Energy invoice for Müller", "Total amount due 12,50 €"].map(String::from);

    let before = engine().has_text_layer(&scan).expect("checked");
    engine().add_text_layer(&scan, &texts, &searchable).expect("written");

    assert!(!before);
    assert!(engine().has_text_layer(&searchable).expect("checked"));
    assert_eq!(engine().page_texts(&searchable).expect("texts"), texts.to_vec());
    let rendered = images::decode(&engine().render_page(&searchable, 0, 300).expect("render").expect("page"))
        .expect("png")
        .to_rgb8();
    assert!(rendered.pixels().all(|pixel| pixel.0 == [255, 255, 255]), "the text layer is invisible");
}

#[test]
fn several_lines_and_extra_texts_go_on_the_page_from_the_top() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[None]);
    let searchable = root.path().join("searchable.pdf");
    let texts = ["first line\nsecond line 日本".to_string(), "next page".to_string()];

    engine().add_text_layer(&scan, &texts, &searchable).expect("written");

    let text = engine().text(&searchable).expect("text");
    assert!(text.contains("first line") && text.contains("second line") && text.contains("next page"), "{text}");
    assert!(!text.contains('日'), "characters the font lacks are left out: {text}");
}

#[test]
fn a_page_number_alone_is_no_text_layer() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("Page 1 of 2"), None]);

    assert!(!engine().has_text_layer(&scan).expect("checked"));
}

#[test]
fn embedded_metadata_reads_back_and_lands_in_xmp() {
    let root = tempfile::tempdir().expect("temp dir");
    let bill = root.path().join("bill.pdf");
    write_text_pdf(&bill, &[Some("bill")]);
    let tagged = root.path().join("tagged.pdf");
    let metadata = PdfMetadata {
        title: Some("Stromrechnung März".into()),
        author: Some("Sample Energy".into()),
        subject: Some("Bills / Power".into()),
        keywords: vec!["Tax".into(), "Home".into()],
        date: Some(PdfDate { year: 2024, month: 3, day: 15 }),
    };

    engine().write_metadata(&bill, &metadata, &tagged).expect("written");

    assert_eq!(engine().read_metadata(&tagged).expect("read"), metadata);
    let document = lopdf::Document::load(&tagged).expect("lopdf");
    let xmp_id = document.catalog().expect("catalog").get(b"Metadata").expect("metadata").as_reference().expect("ref");
    let xmp = document.get_object(xmp_id).expect("object").as_stream().expect("stream").content.clone();
    assert!(String::from_utf8(xmp).expect("utf-8").contains("Stromrechnung März"));
    assert_eq!(engine().page_texts(&tagged).expect("texts"), vec!["bill"]);
}

#[test]
fn metadata_left_empty_removes_what_the_file_had() {
    let root = tempfile::tempdir().expect("temp dir");
    let bill = root.path().join("bill.pdf");
    write_text_pdf(&bill, &[Some("bill")]);
    let tagged = root.path().join("tagged.pdf");
    let cleared = root.path().join("cleared.pdf");
    let metadata =
        PdfMetadata { title: Some("Old title".into()), keywords: vec!["Old".into()], ..PdfMetadata::default() };
    engine().write_metadata(&bill, &metadata, &tagged).expect("written");

    engine().write_metadata(&tagged, &PdfMetadata::default(), &cleared).expect("cleared");

    assert_eq!(engine().read_metadata(&cleared).expect("read"), PdfMetadata::default());
}

#[test]
fn a_page_renders_as_a_small_picture() {
    let root = tempfile::tempdir().expect("temp dir");
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[Some("page")]);

    let picture = engine().render_page(&scan, 0, 120).expect("render").expect("page");

    assert_eq!(picture.mime_type, "image/png");
    let decoded = images::decode(&picture).expect("png");
    assert_eq!(decoded.height(), 120);
    assert!(engine().render_page(&scan, 5, 120).expect("render").is_none());
}

#[test]
fn a_pdf_with_enough_text_is_read_as_text_and_a_scan_as_page_pictures() {
    let root = tempfile::tempdir().expect("temp dir");
    let letter = root.path().join("letter.pdf");
    let long = "Dear customer, this is the invoice for the electricity you used in March 2024.";
    write_text_pdf(&letter, &[Some(long), Some("Second page with the total amount due")]);
    let scan = root.path().join("scan.pdf");
    write_text_pdf(&scan, &[None, None, None]);

    let text = engine().read_content(&letter, false, 30).expect("read");
    let full = engine().read_content(&letter, false, 10_000).expect("read");
    let vision = engine().read_content(&letter, true, 10_000).expect("read");
    let pictures = engine().read_content(&scan, false, 10_000).expect("read");

    assert_eq!(text, DocumentContent::Text(long.chars().take(30).collect()));
    assert!(matches!(full, DocumentContent::Text(ref all) if all.contains("Second page")), "{full:?}");
    assert!(matches!(vision, DocumentContent::PageImages(ref pages) if pages.len() == 2));
    let DocumentContent::PageImages(pages) = pictures else { panic!("scan read as {pictures:?}") };
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].mime_type, "image/jpeg");
    let first = images::decode(&pages[0]).expect("jpeg");
    assert_eq!((first.width(), first.height()), (1131, 1600), "150 dpi A4 limited to 1600 pixels");
}

#[test]
fn a_thumbnail_is_a_small_jpeg_of_the_first_page() {
    let root = tempfile::tempdir().expect("temp dir");
    let letter = root.path().join("letter.pdf");
    write_text_pdf(&letter, &[Some("page")]);

    let thumbnail = engine().thumbnail(&letter, 200).expect("thumbnail").expect("first page");

    assert_eq!(thumbnail.mime_type, "image/jpeg");
    let decoded = images::decode(&thumbnail).expect("jpeg");
    assert_eq!(decoded.height(), 200);
    assert!(matches!(engine().thumbnail(&root.path().join("gone.pdf"), 200), Err(PdfError::NotFound)));
}

#[test]
fn an_empty_page_looks_blank_and_a_written_one_does_not() {
    let root = tempfile::tempdir().expect("temp dir");
    let stack = root.path().join("stack.pdf");
    let line = "Invoice no. 2026-0815 for the maintenance of the heating system in March";
    write_text_pdf(&stack, &[Some(line), None, Some(line)]);

    assert_eq!(engine().blank_pages(&stack).expect("checked"), vec![false, true, false]);
}
