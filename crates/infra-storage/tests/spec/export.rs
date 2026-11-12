use std::fs::{self, File};
use std::io::Read;

use dms_application::port::export::{
    ExportBundle, ExportEntry, ExportFile, ExportTarget, ExportWriter, IndexRow, MissingItem,
};
use dms_infra_storage::FileExportWriter;
use tempfile::TempDir;
use walk::all_names;

use crate::support::{fixture, folder, path, read, value_of, write};

struct Export {
    temp: TempDir,
}

impl Export {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(temp.path().join("library")).expect("library");
        fs::create_dir_all(temp.path().join("outside")).expect("outside");
        Self { temp }
    }

    fn root(&self) -> std::path::PathBuf {
        self.temp.path().join("library")
    }

    fn outside(&self) -> std::path::PathBuf {
        self.temp.path().join("outside")
    }
}

fn row() -> IndexRow {
    IndexRow {
        path: "letters/Grüße.pdf".to_owned(),
        title: "Bill; second \"try\"".to_owned(),
        date: "2024-02-01".to_owned(),
        correspondent: "Müller & Söhne".to_owned(),
        category: "Bills / Power".to_owned(),
        tags: "Tax, Home".to_owned(),
        fields: vec!["12.50 EUR".to_owned()],
        note: "first line\nsecond line".to_owned(),
    }
}

fn bundle() -> ExportBundle {
    ExportBundle {
        entries: vec![ExportEntry { source: path("a/Grüße.pdf"), target: "letters/Grüße.pdf".to_owned() }],
        index: Some(vec![row()]),
        field_labels: vec!["Amount".to_owned()],
        files: Vec::new(),
        missing_items: None,
    }
}

mod walk {
    use std::collections::BTreeSet;
    use std::path::Path;

    pub fn all_names(root: &Path) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for entry in std::fs::read_dir(root).expect("folder") {
            let path = entry.expect("entry").path();
            names.insert(path.file_name().expect("name").to_string_lossy().into_owned());
            if path.is_dir() {
                names.extend(all_names(&path));
            }
        }
        names
    }
}

#[test]
fn a_folder_export_copies_the_files_and_writes_both_index_files_the_library_stays_untouched() {
    let export = Export::new();
    write(&export.root().join("a/Grüße.pdf"), "content");
    let target = ExportTarget::new(export.outside().to_string_lossy(), "Tax 2024", false).expect("target");

    let location = value_of(FileExportWriter.write(&folder(&export.root()), &bundle(), &target));

    let exported = std::path::PathBuf::from(location);
    assert_eq!(exported, export.outside().join("Tax 2024"));
    assert_eq!(read(&exported.join("letters/Grüße.pdf")), "content");
    assert!(exported.join("index.html").exists());
    assert_eq!(read(&export.root().join("a/Grüße.pdf")), "content");
    assert_eq!(all_names(&export.root()), ["Grüße.pdf".to_owned(), "a".to_owned()].into());
}

#[test]
fn the_csv_keeps_umlauts_and_quotes_fields_with_semicolons_quotes_and_line_breaks() {
    let export = Export::new();
    write(&export.root().join("a/Grüße.pdf"), "content");
    let target = ExportTarget::new(export.outside().to_string_lossy(), "Tax", false).expect("target");

    FileExportWriter.write(&folder(&export.root()), &bundle(), &target).expect("exported");

    let bytes = fs::read(export.outside().join("Tax/index.csv")).expect("csv");
    assert_eq!(&bytes[..3], [0xEF, 0xBB, 0xBF]);
    let text = String::from_utf8(bytes[3..].to_vec()).expect("UTF-8");
    assert!(text.starts_with("Path;Title;Date;Correspondent;Category;Tags;Amount;Note\r\n"));
    assert!(text.contains(
        "letters/Grüße.pdf;\"Bill; second \"\"try\"\"\";2024-02-01;Müller & Söhne;Bills / Power;\"Tax, Home\";12.50 EUR;\"first line\nsecond line\"\r\n"
    ));
}

/// The index files the Kotlin version wrote for the same rows, including its HTML indentation.
#[test]
fn the_index_files_have_the_bytes_the_kotlin_version_wrote() {
    let export = Export::new();
    let first = IndexRow {
        path: "letters/Grüße & <Söhne>.pdf".to_owned(),
        fields: vec!["12.50 EUR".to_owned(), String::new()],
        note: "first line\nsecond line\r\nthird".to_owned(),
        ..row()
    };
    let second = IndexRow { path: "a b+c/ä?.pdf".to_owned(), note: String::new(), ..first.clone() };
    let bundle = ExportBundle {
        entries: Vec::new(),
        index: Some(vec![first, second]),
        field_labels: vec!["Amount".to_owned(), "Due; date".to_owned()],
        files: Vec::new(),
        missing_items: None,
    };
    let target = ExportTarget::new(export.outside().to_string_lossy(), "Index", false).expect("target");

    FileExportWriter.write(&folder(&export.root()), &bundle, &target).expect("exported");

    let golden = |name: &str| {
        fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/jackson").join(name))
    };
    assert_eq!(fs::read(export.outside().join("Index/index.csv")).expect("csv"), golden("index.csv").expect("golden"));
    assert_eq!(read(&export.outside().join("Index/index.html")), fixture("jackson/index.html"));
}

#[test]
fn a_zip_export_holds_the_files_and_the_index_and_an_existing_name_is_never_replaced() {
    let export = Export::new();
    write(&export.root().join("a/Grüße.pdf"), "content");
    write(&export.outside().join("Tax.zip"), "older export");
    let target = ExportTarget::new(export.outside().to_string_lossy(), "Tax", true).expect("target");

    let location = value_of(FileExportWriter.write(&folder(&export.root()), &bundle(), &target));

    assert_eq!(location, export.outside().join("Tax (2).zip").to_string_lossy());
    assert_eq!(read(&export.outside().join("Tax.zip")), "older export");
    let mut archive = zip::ZipArchive::new(File::open(&location).expect("zip")).expect("archive");
    let names: std::collections::BTreeSet<String> =
        archive.file_names().map(|name| name.expect("name").into_owned()).collect();
    assert_eq!(names, ["index.csv", "index.html", "letters/Grüße.pdf"].map(str::to_owned).into());
    let mut content = String::new();
    archive.by_name("letters/Grüße.pdf").expect("entry").read_to_string(&mut content).expect("content");
    assert_eq!(content, "content");
    assert!(!all_names(&export.outside()).iter().any(|name| name.starts_with(".dms-tmp-")));
}

#[test]
fn a_target_inside_a_library_folder_is_recognized() {
    let export = Export::new();
    let library = [folder(&export.root())];

    assert!(FileExportWriter.is_inside(&export.root().join("exports").to_string_lossy(), &library));
    assert!(FileExportWriter.is_inside(&export.root().to_string_lossy(), &library));
    assert!(FileExportWriter.is_inside(&export.root().join("exports/../exports").to_string_lossy(), &library));
    assert!(!FileExportWriter.is_inside(&export.outside().to_string_lossy(), &library));
}

#[test]
fn a_text_file_such_as_a_calendar_is_written_under_a_free_name() {
    let export = Export::new();
    write(&export.outside().join("deadlines.ics"), "old");

    let location =
        value_of(FileExportWriter.write_text(&export.outside().to_string_lossy(), "deadlines.ics", "BEGIN:VCALENDAR"));

    assert_eq!(read(std::path::Path::new(&location)), "BEGIN:VCALENDAR");
    assert_eq!(read(&export.outside().join("deadlines.ics")), "old");
}

fn tax_bundle() -> ExportBundle {
    ExportBundle {
        files: vec![ExportFile { target: "Missing items.pdf".to_owned(), content: b"%PDF page".to_vec() }],
        missing_items: Some(vec![MissingItem {
            area: "Donations".to_owned(),
            label: "Spende; Seenotretter".to_owned(),
            from: "Sea Rescue".to_owned(),
            state: "Missing".to_owned(),
            note: String::new(),
        }]),
        ..bundle()
    }
}

#[test]
fn a_tax_export_holds_the_missing_items_page_and_sheet_in_a_folder_and_in_a_zip() {
    let export = Export::new();
    write(&export.root().join("a/Grüße.pdf"), "content");
    let in_folder = ExportTarget::new(export.outside().to_string_lossy(), "Tax", false).expect("target");
    let in_zip = ExportTarget::new(export.outside().to_string_lossy(), "Tax", true).expect("target");

    let folder_location = value_of(FileExportWriter.write(&folder(&export.root()), &tax_bundle(), &in_folder));
    let zip_location = value_of(FileExportWriter.write(&folder(&export.root()), &tax_bundle(), &in_zip));

    let exported = std::path::PathBuf::from(folder_location);
    assert_eq!(read(&exported.join("Missing items.pdf")), "%PDF page");
    let sheet = fs::read(exported.join("Missing items.csv")).expect("sheet");
    let text = String::from_utf8(sheet[3..].to_vec()).expect("UTF-8");
    assert_eq!(text, "Area;Item;From;State;Note\r\nDonations;\"Spende; Seenotretter\";Sea Rescue;Missing;\r\n");
    let mut archive = zip::ZipArchive::new(File::open(&zip_location).expect("zip")).expect("archive");
    assert_eq!(archive.by_index(0).expect("first").name().expect("name"), "Missing items.pdf", "the page comes first");
    let mut content = String::new();
    archive.by_name("Missing items.pdf").expect("page").read_to_string(&mut content).expect("content");
    assert_eq!(content, "%PDF page");
    assert!(archive.by_name("Missing items.csv").is_ok());
}

#[test]
fn an_encrypted_zip_opens_only_with_its_password_and_holds_the_extra_files_without_the_index() {
    let export = Export::new();
    write(&export.root().join("a/Grüße.pdf"), "content");
    let target = ExportTarget::new(export.outside().to_string_lossy(), "Emergency folder", true).expect("target");
    let extra = vec![("Emergency overview.pdf".to_owned(), b"keys are with the neighbour".to_vec())];

    let location = value_of(FileExportWriter.write_encrypted_zip(
        &folder(&export.root()),
        &bundle(),
        &extra,
        &target,
        "correct horse",
    ));

    assert!(location.ends_with("Emergency folder.zip"), "{location}");
    let mut archive = zip::ZipArchive::new(File::open(&location).expect("zip")).expect("archive");
    let mut names: Vec<String> = archive.file_names().map(|name| name.expect("name").into_owned()).collect();
    names.sort();
    assert_eq!(names, vec!["Emergency overview.pdf".to_owned(), "letters/Grüße.pdf".to_owned()]);
    let mut text = String::new();
    archive
        .by_name_decrypt("letters/Grüße.pdf", b"correct horse")
        .expect("decrypted")
        .read_to_string(&mut text)
        .expect("text");
    assert_eq!(text, "content");
    assert!(archive.by_name_decrypt("Emergency overview.pdf", b"wrong password").is_err());
    assert!(archive.by_name("Emergency overview.pdf").is_err(), "no entry reads without the password");
    let raw = fs::read(&location).expect("bytes");
    assert!(!raw.windows(9).any(|window| window == b"neighbour"), "the content is not stored in plain text");
}

#[test]
fn a_file_is_written_under_a_free_name_next_to_an_existing_one() {
    let export = Export::new();
    write(&export.outside().join("overview.pdf"), "older");

    let location = value_of(FileExportWriter.write_file(&export.outside().to_string_lossy(), "overview.pdf", b"%PDF"));

    assert!(location.ends_with("overview (2).pdf"), "{location}");
    assert_eq!(read(&export.outside().join("overview.pdf")), "older");
}
