//! The index of an export as `index.csv` and `index.html`. The CSV is UTF-8 with a byte order mark, separated by
//! `;` and quoted where needed, so spreadsheet applications read umlauts and line breaks right. The HTML is one
//! plain table with links to the files.

use std::fmt::Write as _;

use dms_application::port::export::{ExportBundle, IndexRow, MissingItem};
use dms_domain::text;

const SEPARATOR: &str = ";";
const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];
const FIXED_COLUMNS: [&str; 6] = ["Path", "Title", "Date", "Correspondent", "Category", "Tags"];

pub fn csv(bundle: &ExportBundle, rows: &[IndexRow]) -> Vec<u8> {
    let mut lines = vec![header(bundle)];
    lines.extend(rows.iter().map(values_of));
    csv_of(lines)
}

/// The "Missing items" sheet of a tax export, in the same CSV dialect as the index.
pub fn missing_items_csv(items: &[MissingItem]) -> Vec<u8> {
    let mut lines = vec![["Area", "Item", "From", "State", "Note"].map(str::to_owned).to_vec()];
    lines.extend(items.iter().map(|item| {
        vec![item.area.clone(), item.label.clone(), item.from.clone(), item.state.clone(), item.note.clone()]
    }));
    csv_of(lines)
}

fn csv_of(lines: Vec<Vec<String>>) -> Vec<u8> {
    let mut text = String::new();
    for line in lines {
        let fields: Vec<String> = line.iter().map(|value| csv_field(value)).collect();
        text.push_str(&fields.join(SEPARATOR));
        text.push_str("\r\n");
    }
    [BYTE_ORDER_MARK, text.as_bytes()].concat()
}

pub fn html(bundle: &ExportBundle, rows: &[IndexRow]) -> Vec<u8> {
    let header = header(bundle).iter().fold(String::new(), |mut header, column| {
        let _ = write!(header, "<th>{}</th>", escaped(column));
        header
    });
    let body = rows
        .iter()
        .map(|row| {
            let link = format!("<td><a href=\"{}\">{}</a></td>", escaped(&encoded_path(&row.path)), escaped(&row.path));
            let cells = values_of(row).iter().skip(1).fold(String::new(), |mut cells, value| {
                let _ = write!(cells, "<td>{}</td>", escaped(value));
                cells
            });
            format!("<tr>{link}{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("\n");
    // The Kotlin version built this as an indented raw string and called `trimIndent()` after filling it in.
    let indent = " ".repeat(12);
    let page = [
        "<!DOCTYPE html>",
        "<html><head><meta charset=\"utf-8\"><title>Documents</title>",
        "<style>body{font-family:sans-serif}table{border-collapse:collapse}",
        "td,th{border:1px solid #ccc;padding:4px 8px;vertical-align:top;white-space:pre-wrap}</style>",
        "</head><body><table>",
        &format!("<tr>{header}</tr>"),
        &body,
        "</table></body></html>",
    ]
    .iter()
    .fold(String::from("\n"), |mut page, line| {
        let _ = writeln!(page, "{indent}{line}");
        page
    });
    trim_indent(&format!("{page}        ")).into_bytes()
}

fn header(bundle: &ExportBundle) -> Vec<String> {
    let mut columns: Vec<String> = FIXED_COLUMNS.iter().map(|column| (*column).to_owned()).collect();
    columns.extend(bundle.field_labels.iter().cloned());
    columns.push("Note".to_owned());
    columns
}

fn values_of(row: &IndexRow) -> Vec<String> {
    let mut values = vec![
        row.path.clone(),
        row.title.clone(),
        row.date.clone(),
        row.correspondent.clone(),
        row.category.clone(),
        row.tags.clone(),
    ];
    values.extend(row.fields.iter().cloned());
    values.push(row.note.clone());
    values
}

/// RFC 4180 quoting: a field with the separator, a quote, a comma or a line break is quoted, quotes doubled.
fn csv_field(value: &str) -> String {
    if value.contains([';', '"', '\n', '\r', ',']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Each segment as Java's `URLEncoder.encode` in UTF-8, with `%20` for a space.
fn encoded_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            segment.bytes().fold(String::new(), |mut encoded, byte| {
                if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'*' | b'_') {
                    encoded.push(char::from(byte));
                } else {
                    let _ = write!(encoded, "%{byte:02X}");
                }
                encoded
            })
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Kotlin's `String.trimIndent()`: drops a blank first and last line and the indent common to all other non-blank
/// lines, and joins the lines (split at `\r\n`, `\n` or `\r`) with `\n`.
fn trim_indent(text: &str) -> String {
    let lines: Vec<&str> = text.split("\r\n").flat_map(|part| part.split(['\n', '\r'])).collect();
    let indent_width = |line: &str| line.chars().take_while(|character| text::is_whitespace(*character)).count();
    let common = lines.iter().filter(|line| !text::is_blank(line)).map(|line| indent_width(line)).min().unwrap_or(0);
    let last = lines.len().saturating_sub(1);
    lines
        .iter()
        .enumerate()
        .filter(|(index, line)| !((*index == 0 || *index == last) && text::is_blank(line)))
        .map(|(_, line)| line.chars().skip(common).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_indent_follows_kotlin() {
        assert_eq!(trim_indent("\n    a\n      b\n    "), "a\n  b");
        assert_eq!(trim_indent("\n    a\nb\n    "), "    a\nb");
        assert_eq!(encoded_path("a b+c/ä?.pdf"), "a%20b%2Bc/%C3%A4%3F.pdf");
    }
}
