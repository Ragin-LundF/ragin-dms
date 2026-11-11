use indexmap::IndexMap;
use saphyr_parser::{Event, Parser, ScalarStyle, Tag};

use crate::yaml::{Node, YamlError};

/// Parses the first document of `text`; `None` when there is none (an empty file). Anchors and aliases are not
/// supported: the tool never writes them.
///
/// # Errors
/// Fails for text that is no YAML, a complex mapping key or an alias.
pub fn parse(text: &str) -> Result<Option<Node>, YamlError> {
    // saphyr refuses U+FEFF inside a scalar, which snakeyaml (and so Jackson) reads and writes as text; it is
    // swapped for a noncharacter while parsing and restored in every scalar.
    if text.contains(BOM) && !text.contains(STAND_IN) {
        return parse_events(&text.replace(BOM, &STAND_IN.to_string())).map(|node| node.map(restore_bom));
    }
    parse_events(text)
}

const BOM: char = '\u{feff}';
const STAND_IN: char = '\u{fdd0}';

fn restore_bom(node: Node) -> Node {
    let restore = |text: String| text.replace(STAND_IN, &BOM.to_string());
    match node {
        Node::Text(text) => Node::Text(restore(text)),
        Node::Seq(items) => Node::Seq(items.into_iter().map(restore_bom).collect()),
        Node::Map(entries) => {
            Node::Map(entries.into_iter().map(|(key, value)| (restore(key), restore_bom(value))).collect())
        }
        other => other,
    }
}

fn parse_events(text: &str) -> Result<Option<Node>, YamlError> {
    let mut events = Parser::new_from_str(text);
    loop {
        match next(&mut events)? {
            Event::StreamStart | Event::Nothing => {}
            Event::DocumentStart(_) => return node_from(next(&mut events)?, &mut events).map(Some),
            _ => return Ok(None),
        }
    }
}

/// [`parse`] for the bytes of a file: UTF-8, a byte order mark is skipped.
///
/// # Errors
/// Fails for bytes that are no UTF-8 or no YAML.
pub fn parse_bytes(bytes: &[u8]) -> Result<Option<Node>, YamlError> {
    let text = std::str::from_utf8(bytes).map_err(|error| YamlError(error.to_string()))?;
    parse(text.strip_prefix('\u{feff}').unwrap_or(text))
}

fn next<'input>(events: &mut Parser<'input, saphyr_parser::StrInput<'input>>) -> Result<Event<'input>, YamlError> {
    match events.next() {
        Some(Ok((event, _))) => Ok(event),
        Some(Err(error)) => Err(YamlError(error.to_string())),
        None => Err(YamlError("unexpected end of the document".to_owned())),
    }
}

fn node_from<'input>(
    event: Event<'input>,
    events: &mut Parser<'input, saphyr_parser::StrInput<'input>>,
) -> Result<Node, YamlError> {
    match event {
        Event::Scalar(value, style, _, tag) => Ok(scalar(&value, style, tag.as_deref())),
        Event::SequenceStart(..) => {
            let mut items = Vec::new();
            loop {
                match next(events)? {
                    Event::SequenceEnd => return Ok(Node::Seq(items)),
                    event => items.push(node_from(event, events)?),
                }
            }
        }
        Event::MappingStart(..) => {
            let mut entries = IndexMap::new();
            loop {
                let key = match next(events)? {
                    Event::MappingEnd => return Ok(Node::Map(entries)),
                    Event::Scalar(value, ..) => value.into_owned(),
                    _ => return Err(YamlError("a mapping key must be a scalar".to_owned())),
                };
                let value = node_from(next(events)?, events)?;
                // Like Jackson, a repeated key keeps its first position and takes the last value.
                entries.insert(key, value);
            }
        }
        Event::Alias(_) => Err(YamlError("aliases are not supported".to_owned())),
        other => Err(YamlError(format!("unexpected YAML event {other:?}"))),
    }
}

fn scalar(value: &str, style: ScalarStyle, tag: Option<&Tag>) -> Node {
    match tag.filter(|tag| tag.is_yaml_core_schema()).map(|tag| tag.suffix.as_str()) {
        Some("null") if !value.is_empty() => Node::Null,
        Some("bool") if value.eq_ignore_ascii_case("true") => Node::Bool(true),
        Some("bool") if value.eq_ignore_ascii_case("false") => Node::Bool(false),
        Some("int") if !value.is_empty() => Node::Int(value.to_string()),
        Some("float") if !value.is_empty() => Node::Float(value.to_string()),
        None if style == ScalarStyle::Plain && tag.is_none() => Node::plain(value),
        _ => Node::text(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> Node {
        parse(text).expect("valid YAML").expect("a document")
    }

    #[test]
    fn quoted_scalars_stay_text_and_plain_ones_are_resolved() {
        let node = parsed("a: \"5\"\nb: 5\nc: ~\nd: null\ne: 'true'\nf:\n");

        let Node::Map(entries) = node else { panic!("a mapping") };
        assert_eq!(entries["a"], Node::text("5"));
        assert_eq!(entries["b"], Node::Int("5".to_owned()));
        assert_eq!(entries["c"], Node::text("~"));
        assert_eq!(entries["d"], Node::Null);
        assert_eq!(entries["e"], Node::text("true"));
        assert_eq!(entries["f"], Node::Null);
    }

    #[test]
    fn an_empty_file_has_no_document_and_broken_text_fails() {
        assert_eq!(parse("").expect("empty is valid"), None);
        assert!(parse("formatVersion: [1\n").is_err());
        assert!(parse("a: &x 1\nb: *x\n").is_err());
    }

    #[test]
    fn a_repeated_key_keeps_its_place_and_takes_the_last_value() {
        let Node::Map(entries) = parsed("a: 1\nb: 2\na: 3\n") else { panic!("a mapping") };

        assert_eq!(entries.keys().collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(entries["a"], Node::Int("3".to_owned()));
    }
}
