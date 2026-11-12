use std::path::Path;

use pdfium_render::prelude::{
    PdfDocument, PdfPageContentRegenerationStrategy, PdfPageObjectsCommon, PdfPagePaperSize, PdfPageTextObject,
    PdfPoints,
};

use super::pages::save_new;
use crate::engine::PdfEngine;
use crate::error::PdfError;
use crate::files::FileFacts;
use crate::fonts::{BundledFont, FontStyle, font_of};
use crate::rebuild::FontFamily;

/// One block of a text document; see `dms_application::port::pdf::TextBlock`, which the adapter maps to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBlock {
    Heading(String),
    Paragraph(String),
    KeyValueTable(Vec<(String, String)>),
    Table { header: Vec<String>, rows: Vec<Vec<String>> },
    Spacer,
}

/// A4 in points.
const PAGE_WIDTH: f32 = 595.28;
const PAGE_HEIGHT: f32 = 841.89;
const MARGIN: f32 = 64.0;
const TOP: f32 = PAGE_HEIGHT - MARGIN;
const TEXT_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN;
const BLOCK_GAP: f32 = 6.0;
const COLUMN_GAP: f32 = 12.0;
/// The label column of a key-value table takes at most this share of the width.
const MAX_LABEL_SHARE: f32 = 0.4;

#[derive(Debug, Clone, Copy)]
struct Style {
    size: f32,
    leading: f32,
    bold: bool,
}

const BODY: Style = Style { size: 11.0, leading: 15.0, bold: false };
const LABEL: Style = Style { bold: true, ..BODY };
const HEADING: Style = Style { size: 14.0, leading: 19.0, bold: true };
const CELL: Style = Style { size: 10.0, leading: 13.5, bold: false };
const HEADER_CELL: Style = Style { bold: true, ..CELL };

/// One line of text on a page: its left edge and baseline in points from the bottom left.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlacedText {
    pub text: String,
    pub x: f32,
    pub baseline: f32,
    pub size: f32,
    pub bold: bool,
}

impl PdfEngine {
    /// A new PDF of [`blocks`] on A4 pages in the embedded Liberation Sans, as the text layer uses it, so umlauts,
    /// ß and € stay readable and searchable; characters the font lacks are left out. Lines wrap at the margins and
    /// pages break on their own. [`target`] must not exist yet.
    ///
    /// # Errors
    /// [`PdfError::AlreadyExists`] if [`target`] exists; [`PdfError::Io`] if it cannot be written.
    pub fn write_text_document(self, blocks: &[TextBlock], target: &Path) -> Result<FileFacts, PdfError> {
        save_new(&self.text_document(blocks)?, target)
    }

    /// The PDF [`Self::write_text_document`] writes, as bytes, for a file that goes outside the library.
    ///
    /// # Errors
    /// [`PdfError`] when PDFium fails to build or save the document.
    pub fn text_document_bytes(self, blocks: &[TextBlock]) -> Result<Vec<u8>, PdfError> {
        self.text_document(blocks)?.save_to_bytes().map_err(PdfError::from)
    }

    fn text_document(self, blocks: &[TextBlock]) -> Result<PdfDocument<'static>, PdfError> {
        let pages = layout(blocks);
        let mut document = self.create()?;
        let regular = document.fonts_mut().load_true_type_from_bytes(sans(false).bytes, true)?;
        let bold = document.fonts_mut().load_true_type_from_bytes(sans(true).bytes, true)?;
        let size = PdfPagePaperSize::Custom(PdfPoints::new(PAGE_WIDTH), PdfPoints::new(PAGE_HEIGHT));
        for texts in &pages {
            let mut page = document.pages_mut().create_page_at_end(size)?;
            page.set_content_regeneration_strategy(PdfPageContentRegenerationStrategy::Manual);
            for placed in texts {
                let font = if placed.bold { bold } else { regular };
                let mut object = PdfPageTextObject::new(&document, &placed.text, font, PdfPoints::new(placed.size))?;
                object.translate(PdfPoints::new(placed.x), PdfPoints::new(placed.baseline))?;
                page.objects_mut().add_text_object(object)?;
            }
            page.regenerate_content()?;
        }
        Ok(document)
    }
}

fn sans(bold: bool) -> &'static BundledFont {
    font_of(FontStyle { family: FontFamily::Sans, bold })
}

/// The lines of every page; at least one page, which may be empty.
pub(crate) fn layout(blocks: &[TextBlock]) -> Vec<Vec<PlacedText>> {
    let mut layout = Layout { pages: vec![Vec::new()], top: TOP };
    for block in blocks {
        match block {
            TextBlock::Heading(text) => layout.paragraph(text, HEADING),
            TextBlock::Paragraph(text) => layout.paragraph(text, BODY),
            TextBlock::KeyValueTable(rows) => layout.key_values(rows),
            TextBlock::Table { header, rows } => layout.table(header, rows),
            TextBlock::Spacer => layout.top -= BODY.leading,
        }
    }
    layout.pages
}

/// One cell of a row: where it starts, how wide it is, its text and style.
struct Cell<'a> {
    x: f32,
    width: f32,
    text: &'a str,
    style: Style,
}

/// The pages laid out so far and the top of the next line on the last one.
struct Layout {
    pages: Vec<Vec<PlacedText>>,
    top: f32,
}

impl Layout {
    fn paragraph(&mut self, text: &str, style: Style) {
        for line in text.lines().flat_map(|line| wrap(line, TEXT_WIDTH, style)) {
            self.make_room(style.leading);
            self.place(line, MARGIN, style);
            self.top -= style.leading;
        }
        self.top -= BLOCK_GAP;
    }

    fn key_values(&mut self, rows: &[(String, String)]) {
        let widest = rows.iter().map(|(label, _)| sans(true).width_at_1(label) * LABEL.size).fold(0.0, f32::max);
        let label_width = (widest + COLUMN_GAP).min(TEXT_WIDTH * MAX_LABEL_SHARE);
        for (label, value) in rows {
            self.row(&[
                Cell { x: MARGIN, width: label_width - COLUMN_GAP, text: label, style: LABEL },
                Cell { x: MARGIN + label_width, width: TEXT_WIDTH - label_width, text: value, style: BODY },
            ]);
        }
        self.top -= BLOCK_GAP;
    }

    /// Equal columns; the header comes again at the top of every page the table runs onto.
    fn table(&mut self, header: &[String], rows: &[Vec<String>]) {
        let columns = rows.iter().map(Vec::len).chain([header.len()]).max().unwrap_or(0);
        if columns == 0 {
            return;
        }
        #[allow(clippy::cast_precision_loss)] // a table has far fewer than 2^24 columns
        let width = TEXT_WIDTH / columns as f32;
        let header = cells(header, width, HEADER_CELL);
        self.row(&header);
        for row in rows {
            let row = cells(row, width, CELL);
            if !self.fits(height_of(&row)) && !header.is_empty() {
                self.new_page();
                self.row(&header);
            }
            self.row(&row);
        }
        self.top -= BLOCK_GAP;
    }

    /// A row of cells side by side, kept together on one page.
    // shortcut: a row taller than a page runs over the bottom margin; split rows if tables ever hold such cells.
    fn row(&mut self, cells: &[Cell]) {
        if cells.is_empty() {
            return;
        }
        self.make_room(height_of(cells));
        let start = self.top;
        let mut bottom = start;
        for cell in cells {
            self.top = start;
            for line in cell_lines(cell) {
                self.place(line, cell.x, cell.style);
                self.top -= cell.style.leading;
            }
            bottom = bottom.min(self.top);
        }
        self.top = bottom;
    }

    fn fits(&self, height: f32) -> bool {
        self.top - height >= MARGIN
    }

    /// Starts a new page unless [`height`] still fits or nothing was placed on this page yet.
    fn make_room(&mut self, height: f32) {
        if !self.fits(height) && self.top < TOP {
            self.new_page();
        }
    }

    fn new_page(&mut self) {
        self.pages.push(Vec::new());
        self.top = TOP;
    }

    fn place(&mut self, text: String, x: f32, style: Style) {
        if text.is_empty() {
            return;
        }
        let placed = PlacedText { text, x, baseline: self.top - style.size, size: style.size, bold: style.bold };
        self.pages.last_mut().expect("invariant: the layout starts with a page").push(placed);
    }
}

/// One cell per text, side by side in columns of [`width`].
fn cells(texts: &[String], width: f32, style: Style) -> Vec<Cell<'_>> {
    texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            #[allow(clippy::cast_precision_loss)] // a table has far fewer than 2^24 columns
            let x = MARGIN + index as f32 * width;
            Cell { x, width: width - COLUMN_GAP, text: text.as_str(), style }
        })
        .collect()
}

fn cell_lines(cell: &Cell) -> Vec<String> {
    cell.text.lines().flat_map(|line| wrap(line, cell.width, cell.style)).collect()
}

fn height_of(cells: &[Cell]) -> f32 {
    #[allow(clippy::cast_precision_loss)] // a cell has far fewer than 2^24 lines
    let height = |cell: &Cell| cell_lines(cell).len().max(1) as f32 * cell.style.leading;
    cells.iter().map(height).fold(0.0, f32::max)
}

/// [`text`] broken into lines of at most [`width`] points at word boundaries; a word longer than a line is broken
/// between characters. Characters the font lacks and control characters are left out. At least one line.
fn wrap(text: &str, width: f32, style: Style) -> Vec<String> {
    let font = sans(style.bold);
    let fits = |candidate: &str| font.width_at_1(candidate) * style.size <= width;
    let shown: String = text
        .chars()
        .map(|character| if character == '\t' { ' ' } else { character })
        .filter(|character| !character.is_control() && font.has_glyph(*character))
        .collect();
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in shown.split_whitespace() {
        let candidate = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
        if fits(&candidate) {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for character in word.chars() {
            line.push(character);
            if !fits(&line) && line.chars().count() > 1 {
                line.pop();
                lines.push(std::mem::replace(&mut line, character.to_string()));
            }
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts_of(page: &[PlacedText]) -> Vec<&str> {
        page.iter().map(|placed| placed.text.as_str()).collect()
    }

    #[test]
    fn long_text_wraps_inside_the_margins_and_an_overlong_word_is_broken() {
        let lines = wrap(&format!("{} {}", "Wort ".repeat(40), "x".repeat(200)), TEXT_WIDTH, BODY);

        assert!(lines.len() > 3, "{lines:?}");
        assert!(lines.iter().all(|line| sans(false).width_at_1(line) * BODY.size <= TEXT_WIDTH), "{lines:?}");
        assert_eq!(lines.concat().matches('x').count(), 200);
        assert_eq!(wrap("", TEXT_WIDTH, BODY), vec![String::new()]);
        assert_eq!(wrap("a\u{7}b 日本", TEXT_WIDTH, BODY), vec!["ab".to_owned()]);
    }

    #[test]
    fn pages_break_when_full_and_lines_run_from_the_top_down() {
        let paragraph = (1..=120).map(|line| format!("Line {line}")).collect::<Vec<_>>().join("\n");

        let pages = layout(&[TextBlock::Heading("Title".into()), TextBlock::Paragraph(paragraph)]);

        assert_eq!(pages.len(), 3);
        assert_eq!(texts_of(&pages[0])[..2], ["Title", "Line 1"]);
        assert!(pages.iter().flatten().all(|placed| placed.baseline >= MARGIN - BODY.size && placed.baseline < TOP));
        assert!(pages[0][1].baseline < pages[0][0].baseline);
        assert_eq!(texts_of(&pages[2]).last(), Some(&"Line 120"));
    }

    #[test]
    fn a_table_repeats_its_header_on_the_next_page_and_keeps_rows_together() {
        let rows: Vec<Vec<String>> = (1..=80).map(|row| vec![format!("Item {row}"), "12,30 €".to_owned()]).collect();

        let pages = layout(&[TextBlock::Table { header: vec!["Item".into(), "Amount".into()], rows }]);

        assert_eq!(pages.len(), 2);
        assert_eq!(texts_of(&pages[1])[..2], ["Item", "Amount"]);
        assert!(pages[1][0].bold && !pages[1][2].bold);
        assert_eq!(pages[1][2].baseline, pages[1][3].baseline, "the cells of a row share their line");
    }

    #[test]
    fn key_value_rows_put_the_value_beside_its_bold_label() {
        let pages = layout(&[TextBlock::KeyValueTable(vec![("Name".into(), "Erika Müller".into())])]);

        let [label, value] = &pages[0][..] else { panic!("two texts: {:?}", pages[0]) };
        assert!(label.bold && !value.bold);
        assert_eq!(label.baseline, value.baseline);
        assert!(value.x > label.x);
    }

    #[test]
    fn an_empty_document_is_one_empty_page() {
        assert_eq!(layout(&[]), vec![Vec::new()]);
        assert_eq!(layout(&[TextBlock::Table { header: Vec::new(), rows: Vec::new() }]).len(), 1);
    }
}
