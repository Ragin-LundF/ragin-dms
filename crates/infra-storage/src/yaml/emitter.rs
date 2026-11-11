//! Writes a [`Node`] tree byte for byte like Jackson 3 does with the settings of the Kotlin version: Jackson
//! chooses the requested style per scalar ([`scalar_style`]), and this is a port of the parts of the
//! snakeyaml-engine 3.0 `Emitter` that block-style output uses (default settings: indent 2, width 80, split
//! lines, simple keys up to 128 characters, sequences in a mapping not indented).
//!
//! Columns and positions count UTF-16 code units, as in Java.

use indexmap::IndexMap;

use crate::yaml::Node;
use crate::yaml::scalar_style::{self, RequestedStyle};

const BEST_INDENT: usize = 2;
const BEST_WIDTH: usize = 80;
const MAX_SIMPLE_KEY_LENGTH: usize = 128;
const NEWLINE: u16 = b'\n' as u16;
const SPACE: u16 = b' ' as u16;

/// The document text of `node`, ending with a line break, without a `---` start marker.
pub fn emit(node: &Node) -> String {
    let mut emitter = Emitter::default();
    emitter.node(node, false, false);
    emitter.write_indent();
    emitter.out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
}

#[allow(clippy::struct_excessive_bools)] // the state of the Java emitter, kept as it is
struct Emitter {
    out: String,
    indent: Option<usize>,
    indents: Vec<Option<usize>>,
    column: usize,
    whitespace: bool,
    indention: bool,
    mapping_context: bool,
    simple_key_context: bool,
}

impl Default for Emitter {
    fn default() -> Self {
        Self {
            out: String::new(),
            indent: None,
            indents: Vec::new(),
            column: 0,
            whitespace: true,
            indention: true,
            mapping_context: false,
            simple_key_context: false,
        }
    }
}

impl Emitter {
    fn node(&mut self, node: &Node, mapping: bool, simple_key: bool) {
        self.mapping_context = mapping;
        self.simple_key_context = simple_key;
        match node {
            Node::Seq(items) if items.is_empty() => self.empty_flow("[", "]"),
            Node::Map(entries) if entries.is_empty() => self.empty_flow("{", "}"),
            Node::Seq(items) => self.block_sequence(items),
            Node::Map(entries) => self.block_mapping(entries),
            scalar => {
                let (text, requested) = scalar_text(scalar);
                self.scalar(&text, requested);
            }
        }
    }

    fn empty_flow(&mut self, open: &str, close: &str) {
        self.write_indicator(open, true, true, false);
        self.write_indicator(close, false, false, false);
    }

    fn block_sequence(&mut self, items: &[Node]) {
        let indentless = self.mapping_context && !self.indention;
        self.increase_indent(false, indentless);
        for item in items {
            self.write_indent();
            self.write_indicator("-", true, false, true);
            self.node(item, false, false);
        }
        self.pop_indent();
    }

    fn block_mapping(&mut self, entries: &IndexMap<String, Node>) {
        self.increase_indent(false, false);
        for (key, value) in entries {
            self.write_indent();
            let requested = scalar_style::for_key(key);
            if is_simple_key(key) {
                self.node_key(key, requested, true);
                self.write_indicator(":", false, false, false);
            } else {
                self.write_indicator("?", true, false, true);
                self.node_key(key, requested, false);
                self.write_indent();
                self.write_indicator(":", true, false, true);
            }
            self.node(value, true, false);
        }
        self.pop_indent();
    }

    fn node_key(&mut self, key: &str, requested: RequestedStyle, simple: bool) {
        self.mapping_context = true;
        self.simple_key_context = simple;
        self.scalar(key, requested);
    }

    fn scalar(&mut self, text: &str, requested: RequestedStyle) {
        let units: Vec<u16> = text.encode_utf16().collect();
        let analysis = Analysis::of(&units);
        let style = self.choose_style(requested, &analysis);
        self.increase_indent(true, false);
        let split = !self.simple_key_context;
        match style {
            Style::Plain => self.write_plain(&units, split),
            Style::SingleQuoted => self.write_single_quoted(&units, split),
            Style::DoubleQuoted => self.write_double_quoted(&units, split),
            Style::Literal => self.write_literal(&units),
        }
        self.pop_indent();
    }

    /// `Emitter.chooseScalarStyle` (block context only).
    fn choose_style(&self, requested: RequestedStyle, analysis: &Analysis) -> Style {
        match requested {
            RequestedStyle::DoubleQuoted => return Style::DoubleQuoted,
            RequestedStyle::Plain => {
                if !(self.simple_key_context && (analysis.empty || analysis.multiline)) && analysis.allow_block_plain {
                    return Style::Plain;
                }
                if analysis.allow_single_quoted && !(self.simple_key_context && analysis.multiline) {
                    return Style::SingleQuoted;
                }
            }
            RequestedStyle::Literal => {
                if !self.simple_key_context && analysis.allow_block {
                    return Style::Literal;
                }
            }
        }
        Style::DoubleQuoted
    }

    fn increase_indent(&mut self, flow: bool, indentless: bool) {
        self.indents.push(self.indent);
        self.indent = match self.indent {
            None if flow => Some(BEST_INDENT),
            None => Some(0),
            Some(indent) if !indentless => Some(indent + BEST_INDENT),
            current => current,
        };
    }

    fn pop_indent(&mut self) {
        self.indent = self.indents.pop().flatten();
    }

    fn write_indicator(&mut self, indicator: &str, need_whitespace: bool, whitespace: bool, indentation: bool) {
        if !self.whitespace && need_whitespace {
            self.column += 1;
            self.out.push(' ');
        }
        self.whitespace = whitespace;
        self.indention = self.indention && indentation;
        self.column += indicator.encode_utf16().count();
        self.out.push_str(indicator);
    }

    fn write_indent(&mut self) {
        let indent = self.indent.unwrap_or(0);
        if !self.indention || self.column > indent || (self.column == indent && !self.whitespace) {
            self.write_line_break();
        }
        self.write_whitespace(indent.saturating_sub(self.column));
    }

    fn write_whitespace(&mut self, length: usize) {
        if length == 0 {
            return;
        }
        self.whitespace = true;
        self.out.extend(std::iter::repeat_n(' ', length));
        self.column += length;
    }

    fn write_line_break(&mut self) {
        self.whitespace = true;
        self.indention = true;
        self.column = 0;
        self.out.push('\n');
    }

    fn write_units(&mut self, units: &[u16]) {
        self.column += units.len();
        self.out.push_str(&String::from_utf16_lossy(units));
    }

    fn write_breaks(&mut self, breaks: &[u16]) {
        for _ in breaks {
            self.write_line_break();
        }
    }

    fn write_plain(&mut self, text: &[u16], split: bool) {
        if text.is_empty() {
            return;
        }
        if !self.whitespace {
            self.column += 1;
            self.out.push(' ');
        }
        self.whitespace = false;
        self.indention = false;
        let (mut spaces, mut breaks, mut start) = (false, false, 0);
        for end in 0..=text.len() {
            let ch = text.get(end).copied();
            if spaces {
                if ch != Some(SPACE) {
                    if start + 1 == end && self.column > BEST_WIDTH && split {
                        self.write_indent();
                        self.whitespace = false;
                        self.indention = false;
                    } else {
                        self.write_units(&text[start..end]);
                    }
                    start = end;
                }
            } else if breaks {
                if ch != Some(NEWLINE) {
                    if text[start] == NEWLINE {
                        self.write_line_break();
                    }
                    self.write_breaks(&text[start..end]);
                    self.write_indent();
                    self.whitespace = false;
                    self.indention = false;
                    start = end;
                }
            } else if matches!(ch, None | Some(NEWLINE | SPACE)) {
                self.write_units(&text[start..end]);
                start = end;
            }
            if let Some(ch) = ch {
                spaces = ch == SPACE;
                breaks = ch == NEWLINE;
            }
        }
    }

    fn write_single_quoted(&mut self, text: &[u16], split: bool) {
        self.write_indicator("'", true, false, false);
        let (mut spaces, mut breaks, mut start) = (false, false, 0);
        for end in 0..=text.len() {
            let ch = text.get(end).copied();
            if spaces {
                if ch != Some(SPACE) {
                    if start + 1 == end && self.column > BEST_WIDTH && split && start != 0 && end != text.len() {
                        self.write_indent();
                    } else {
                        self.write_units(&text[start..end]);
                    }
                    start = end;
                }
            } else if breaks {
                if ch != Some(NEWLINE) {
                    if text[start] == NEWLINE {
                        self.write_line_break();
                    }
                    self.write_breaks(&text[start..end]);
                    self.write_indent();
                    start = end;
                }
            } else if matches!(ch, None | Some(NEWLINE | SPACE | 0x27)) && start < end {
                self.write_units(&text[start..end]);
                start = end;
            }
            if ch == Some(0x27) {
                self.column += 2;
                self.out.push_str("''");
                start = end + 1;
            }
            if let Some(ch) = ch {
                spaces = ch == SPACE;
                breaks = ch == NEWLINE;
            }
        }
        self.write_indicator("'", false, false, false);
    }

    fn write_double_quoted(&mut self, text: &[u16], split: bool) {
        self.write_indicator("\"", true, false, false);
        let (mut start, mut end) = (0, 0);
        while end <= text.len() {
            let ch = text.get(end).copied();
            if ch.is_none_or(needs_escape_check) {
                if start < end {
                    self.write_units(&text[start..end]);
                    start = end;
                }
                if let Some(ch) = ch {
                    let (data, width) = escaped(text, &mut end, ch);
                    self.column += width;
                    self.out.push_str(&data);
                    start = end + 1;
                }
            }
            if 0 < end
                && end + 1 < text.len()
                && (ch == Some(SPACE) || start >= end)
                // Java's `column + (end - start)`, where `end - start` is -1 right after an escape.
                && (self.column + end + 1).saturating_sub(start) > BEST_WIDTH + 1
                && split
            {
                if start < end {
                    self.write_units(&text[start..end]);
                    start = end;
                }
                self.column += 1;
                self.out.push('\\');
                self.write_indent();
                self.whitespace = false;
                self.indention = false;
                if text[start] == SPACE {
                    self.column += 1;
                    self.out.push('\\');
                }
            }
            end += 1;
        }
        self.write_indicator("\"", false, false, false);
    }

    fn write_literal(&mut self, text: &[u16]) {
        let hints = block_hints(text);
        self.write_indicator(&format!("|{hints}"), true, false, false);
        self.write_line_break();
        let (mut breaks, mut start) = (true, 0);
        for end in 0..=text.len() {
            let ch = text.get(end).copied();
            if breaks {
                if ch != Some(NEWLINE) {
                    self.write_breaks(&text[start..end]);
                    if ch.is_some() {
                        self.write_indent();
                    }
                    start = end;
                }
            } else if matches!(ch, None | Some(NEWLINE)) {
                self.out.push_str(&String::from_utf16_lossy(&text[start..end]));
                if ch.is_none() {
                    self.write_line_break();
                }
                start = end;
            }
            if let Some(ch) = ch {
                breaks = ch == NEWLINE;
            }
        }
    }
}

/// How Jackson writes a non-string scalar, or the style it asks for a string.
fn scalar_text(node: &Node) -> (String, RequestedStyle) {
    match node {
        Node::Null => ("null".to_owned(), RequestedStyle::Plain),
        Node::Bool(value) => (value.to_string(), RequestedStyle::Plain),
        Node::Int(text) if text == "-0" => ("0".to_owned(), RequestedStyle::Plain),
        Node::Int(text) => (text.clone(), RequestedStyle::Plain),
        Node::Float(text) => {
            let value = match text.as_str() {
                ".inf" => f64::INFINITY,
                "-.inf" => f64::NEG_INFINITY,
                ".nan" => f64::NAN,
                other => other.parse().unwrap_or(f64::NAN),
            };
            (scalar_style::java_double(value), RequestedStyle::Plain)
        }
        Node::Text(text) => (text.clone(), scalar_style::for_value(text)),
        Node::Seq(_) | Node::Map(_) => unreachable!("collections are no scalars"),
    }
}

/// `Emitter.checkSimpleKey` for a scalar key.
fn is_simple_key(key: &str) -> bool {
    !key.is_empty() && key.encode_utf16().count() < MAX_SIMPLE_KEY_LENGTH && !key.contains('\n')
}

fn needs_escape_check(unit: u16) -> bool {
    matches!(unit, 0x22 | 0x5C | 0x85 | 0x2028 | 0x2029 | 0xFEFF) || !(0x20..=0x7E).contains(&unit)
}

/// The escape or literal text for `ch` at `*end` in a double-quoted scalar, and its width; moves `*end` past a
/// surrogate pair.
fn escaped(text: &[u16], end: &mut usize, ch: u16) -> (String, usize) {
    let replacement = match ch {
        0x00 => Some("0"),
        0x07 => Some("a"),
        0x08 => Some("b"),
        0x09 => Some("t"),
        0x0A => Some("n"),
        0x0B => Some("v"),
        0x0C => Some("f"),
        0x0D => Some("r"),
        0x1B => Some("e"),
        0x22 => Some("\""),
        0x5C => Some("\\"),
        0x85 => Some("N"),
        0xA0 => Some("_"),
        _ => None,
    };
    if let Some(replacement) = replacement {
        return (format!("\\{replacement}"), 1 + replacement.len());
    }
    let pair = (0xD800..0xDC00).contains(&ch) && *end + 1 < text.len();
    let code_point = if pair {
        0x10000 + ((u32::from(ch) - 0xD800) << 10) + (u32::from(text[*end + 1]) - 0xDC00)
    } else {
        u32::from(ch)
    };
    if is_printable(code_point) {
        let width = if code_point >= 0x10000 { 2 } else { 1 };
        if width == 2 {
            *end += 1;
        }
        let data = char::from_u32(code_point).map(String::from).unwrap_or_default();
        return (data, width);
    }
    let data = if ch <= 0xFF {
        format!("\\x{ch:02x}")
    } else if code_point >= 0x10000 {
        *end += 1;
        format!("\\U{code_point:08x}")
    } else {
        format!("\\u{ch:04x}")
    };
    let width = data.len();
    (data, width)
}

/// `StreamReader.isPrintable`.
fn is_printable(c: u32) -> bool {
    (0x20..=0x7E).contains(&c)
        || matches!(c, 0x9 | 0xA | 0xD | 0x85)
        || (0xA0..=0xD7FF).contains(&c)
        || (0xE000..=0xFFFD).contains(&c)
        || (0x10000..=0x10_FFFF).contains(&c)
}

/// `Emitter.determineBlockHints`: an indentation hint for leading blanks, `-` without and `+` with several trailing
/// line breaks.
fn block_hints(text: &[u16]) -> String {
    let mut hints = String::new();
    if matches!(text.first(), Some(&(NEWLINE | SPACE))) {
        hints.push_str(&BEST_INDENT.to_string());
    }
    let last = text.last().copied();
    if last != Some(NEWLINE) {
        hints.push('-');
    } else if text.len() == 1 || text[text.len() - 2] == NEWLINE {
        hints.push('+');
    }
    hints
}

/// `Emitter.analyzeScalar`, reduced to the flags block-style output needs.
#[allow(clippy::struct_excessive_bools)]
struct Analysis {
    empty: bool,
    multiline: bool,
    allow_block_plain: bool,
    allow_single_quoted: bool,
    allow_block: bool,
}

impl Analysis {
    fn of(text: &[u16]) -> Self {
        if text.is_empty() {
            return Self {
                empty: true,
                multiline: false,
                allow_block_plain: true,
                allow_single_quoted: true,
                allow_block: false,
            };
        }
        let code_points: Vec<u32> =
            char::decode_utf16(text.iter().copied()).map(|decoded| decoded.map_or(0xFFFD, u32::from)).collect();
        let mut flags = Flags::default();
        if starts_with(text, "---") || starts_with(text, "...") {
            flags.block_indicators = true;
        }
        let blank = |c: u32| c == 0 || c == 0x20 || c == 0x09;
        let mut preceded_by_whitespace = true;
        let mut followed_by_whitespace =
            code_points.len() == 1 || code_points.get(1).is_some_and(|&c| blank(c) || c == 0x0A || c == 0x0D);
        let (mut previous_space, mut previous_break) = (false, false);
        for (index, &c) in code_points.iter().enumerate() {
            let is_break = c == 0x0A;
            if index == 0 {
                if "#,[]{}&*!|>'\"%@`".chars().any(|indicator| u32::from(indicator) == c) {
                    flags.block_indicators = true;
                }
                if (c == u32::from('?') || c == u32::from(':') || c == u32::from('-')) && followed_by_whitespace {
                    flags.block_indicators = true;
                }
            } else if (c == u32::from(':') && followed_by_whitespace) || (c == u32::from('#') && preceded_by_whitespace)
            {
                flags.block_indicators = true;
            }
            flags.line_breaks |= is_break;
            if !is_break && !(0x20..=0x7E).contains(&c) && !is_unicode_printable(c) {
                flags.special_characters = true;
            }
            let last = index + 1 == code_points.len();
            if c == 0x20 {
                flags.leading_space |= index == 0;
                flags.trailing_space |= last;
                flags.break_space |= previous_break;
                (previous_space, previous_break) = (true, false);
            } else if is_break {
                flags.leading_break |= index == 0;
                flags.trailing_break |= last;
                flags.space_break |= previous_space;
                (previous_space, previous_break) = (false, true);
            } else {
                (previous_space, previous_break) = (false, false);
            }
            preceded_by_whitespace = blank(c) || is_break;
            followed_by_whitespace = code_points.get(index + 2).is_none_or(|&next| blank(next) || is_break);
        }
        flags.into_analysis()
    }
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
struct Flags {
    block_indicators: bool,
    line_breaks: bool,
    special_characters: bool,
    leading_space: bool,
    leading_break: bool,
    trailing_space: bool,
    trailing_break: bool,
    break_space: bool,
    space_break: bool,
}

impl Flags {
    fn into_analysis(self) -> Analysis {
        let mut allow_block_plain =
            !(self.leading_space || self.leading_break || self.trailing_space || self.trailing_break);
        let mut allow_single_quoted = true;
        let mut allow_block = !self.trailing_space;
        if self.break_space {
            allow_block_plain = false;
            allow_single_quoted = false;
        }
        if self.space_break || self.special_characters {
            allow_block_plain = false;
            allow_single_quoted = false;
            allow_block = false;
        }
        if self.block_indicators {
            allow_block_plain = false;
        }
        Analysis { empty: false, multiline: self.line_breaks, allow_block_plain, allow_single_quoted, allow_block }
    }
}

fn is_unicode_printable(c: u32) -> bool {
    c == 0x85 || (0xA0..=0xD7FF).contains(&c) || (0xE000..=0xFFFD).contains(&c) || (0x10000..=0x10_FFFF).contains(&c)
}

fn starts_with(text: &[u16], prefix: &str) -> bool {
    let prefix: Vec<u16> = prefix.encode_utf16().collect();
    text.starts_with(&prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::parse;

    macro_rules! golden {
        ($name:literal) => {
            ($name, include_str!(concat!("../../tests/fixtures/jackson/", $name)))
        };
    }

    /// Every file Jackson wrote parses into a tree that is written back to the same bytes: quoting, wrapped lines,
    /// block scalars, long keys, empty collections and numbers.
    #[test]
    fn jackson_output_is_written_back_byte_for_byte() {
        let goldens = [
            golden!("scalars.yaml"),
            golden!("saved-odd-catalog.yaml"),
            golden!("saved-future-fields.yaml"),
            golden!("saved-format-1-v1.1.yaml"),
            golden!("settings-default.yaml"),
            golden!("settings-full.yaml"),
            golden!("tasks.yaml"),
            golden!("tasks-empty.yaml"),
            golden!("activity.yaml"),
            golden!("journal.yaml"),
            golden!("plan.yaml"),
            golden!("scan-cache.yaml"),
        ];
        for (name, golden) in goldens {
            let node = parse(golden).expect("valid YAML").expect("a document");
            assert_eq!(emit(&node), golden, "{name}");
        }
    }
}
