//! Understands the model's answer to "return only the tags, comma-separated". Models still add a lead-in ("Tags:",
//! "Here are the tags:"), code fences, bullets, numbering, quotes or hashes, or put one tag per line; all of that is
//! removed, and fragments longer than [`MAX_WORDS`] words are no tags. Existing tags keep their spelling, and at most
//! [`MAX_TAGS`] tags are kept.

use dms_domain::taxonomy::TagName;
use dms_domain::text;
use indexmap::IndexSet;

const MAX_TAGS: usize = 8;
const MAX_WORDS: usize = 3;
const FENCE: &str = "```";

pub(crate) fn parse(answer: &str, existing_tags: &IndexSet<TagName>) -> Vec<TagName> {
    let joined = kotlin_lines(answer)
        .filter(|line| !text::trim(line).starts_with(FENCE))
        .map(|line| line.split_once(':').map_or(line, |(_, rest)| rest))
        .collect::<Vec<_>>()
        .join("\n");
    let mut tags: Vec<TagName> = Vec::new();
    for raw in joined.split([',', ';', '\n']).map(|part| without_trailing(without_leading(part))) {
        if raw.is_empty() || raw.split(' ').count() > MAX_WORDS {
            continue;
        }
        let Some(tag) = TagName::parse(raw) else {
            continue;
        };
        let tag = existing_tags.get(&tag).cloned().unwrap_or(tag);
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags.truncate(MAX_TAGS);
    tags
}

/// Lines split at `\r\n`, `\n` and `\r`, as Kotlin's `lines()`.
fn kotlin_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).flat_map(|line| line.split('\r'))
}

/// Repeatedly drops leading spaces, bullets, quotes and hashes, and numbering such as `1.` or `2)` before a space.
fn without_leading(part: &str) -> &str {
    let mut rest = part;
    loop {
        let stripped = rest.trim_start_matches(|c: char| text::is_regex_space(c) || "-*•#>\"'`".contains(c));
        let stripped = without_numbering(stripped).unwrap_or(stripped);
        if stripped.len() == rest.len() {
            return rest;
        }
        rest = stripped;
    }
}

/// `\d+[.)]\s+` at the start removed, or `None` if there is no such numbering.
fn without_numbering(part: &str) -> Option<&str> {
    let after_digits = part.trim_start_matches(|c: char| c.is_ascii_digit());
    if after_digits.len() == part.len() {
        return None;
    }
    let after_mark = after_digits.strip_prefix(['.', ')'])?;
    let after_space = after_mark.trim_start_matches(text::is_regex_space);
    (after_space.len() < after_mark.len()).then_some(after_space)
}

fn without_trailing(part: &str) -> &str {
    part.trim_end_matches(|c: char| text::is_regex_space(c) || "\"'`.*".contains(c))
}
