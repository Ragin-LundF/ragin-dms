//! The text of a Word, `PowerPoint` or Excel file: the paragraphs of a document, the text of every slide, or every
//! sheet's name and rows with tab-separated cells.

use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use calamine::{Reader, Xlsx};
use dms_domain::text;
use quick_xml::events::Event;
use zip::ZipArchive;

const WORD_BODY: &str = "word/document.xml";
const SLIDE_PREFIX: &str = "ppt/slides/slide";
const XML_SUFFIX: &str = ".xml";

/// The whole text, trimmed, or an empty string when there is none.
///
/// # Errors
/// The reason, when the file is missing, damaged or no Office file.
pub(crate) fn text(file: &Path) -> Result<String, String> {
    let extension = text::lowercase(&file.extension().unwrap_or_default().to_string_lossy());
    let text = if extension == "xlsx" {
        spreadsheet_text(file)?
    } else {
        let mut archive =
            ZipArchive::new(File::open(file).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
        if extension == "pptx" { slides_text(&mut archive)? } else { xml_text(&mut archive, WORD_BODY)? }
    };
    Ok(text::trim(&text).to_owned())
}

fn slides_text<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<String, String> {
    let mut slides: Vec<(u32, String)> = archive
        .file_names()
        .filter_map(|name| {
            let number = name.strip_prefix(SLIDE_PREFIX)?.strip_suffix(XML_SUFFIX)?.parse().ok()?;
            Some((number, name.to_owned()))
        })
        .collect();
    slides.sort_unstable();
    let mut text = String::new();
    for (_, name) in slides {
        text.push_str(&xml_text(archive, &name)?);
    }
    Ok(text)
}

/// The text runs (`w:t`, `a:t`) of one XML part; paragraphs end with a line break, tabs and breaks are kept.
fn xml_text<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<String, String> {
    let entry = archive.by_name(name).map_err(|error| format!("{name}: {error}"))?;
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(entry));
    let mut buffer = Vec::new();
    let mut text = String::new();
    let mut in_run = false;
    loop {
        let event = reader.read_event_into(&mut buffer).map_err(|error| error.to_string())?;
        match event {
            Event::Start(element) if element.local_name().as_ref() == b"t" => in_run = true,
            Event::End(element) => match element.local_name().as_ref() {
                b"t" => in_run = false,
                b"p" => text.push('\n'),
                _ => {}
            },
            Event::Empty(element) => match element.local_name().as_ref() {
                b"tab" => text.push('\t'),
                b"br" | b"cr" => text.push('\n'),
                _ => {}
            },
            Event::Text(run) if in_run => text.push_str(&run.decode().map_err(|error| error.to_string())?),
            Event::GeneralRef(reference) if in_run => {
                let name = reference.decode().map_err(|error| error.to_string())?;
                let reference = format!("&{name};");
                text.push_str(&quick_xml::escape::unescape(&reference).map_err(|error| error.to_string())?);
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(text)
}

fn spreadsheet_text(file: &Path) -> Result<String, String> {
    let mut workbook: Xlsx<_> =
        calamine::open_workbook(file).map_err(|error: calamine::XlsxError| error.to_string())?;
    let mut text = String::new();
    for name in workbook.sheet_names() {
        let range = workbook.worksheet_range(&name).map_err(|error| error.to_string())?;
        text.push_str(&name);
        text.push('\n');
        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(ToString::to_string).collect();
            text.push_str(&cells.join("\t"));
            text.push('\n');
        }
    }
    Ok(text)
}
