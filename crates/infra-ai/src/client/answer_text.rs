use dms_domain::text;

const THINK_OPEN: &str = "<think>";
const THINK_CLOSE: &str = "</think>";

/// Cleans what a model answered: reasoning models put their thoughts in `<think>` blocks before the answer, which
/// the user should not see, and some models mark words **bold** despite being asked not to.
pub(crate) struct AnswerText;

impl AnswerText {
    /// The answer without Markdown emphasis, which the app shows as plain text.
    pub(crate) fn plain(content: &str) -> String {
        Self::clean(content).replace("**", "")
    }

    /// Without `<think>…</think>` blocks and without anything up to a lone closing `</think>`, trimmed.
    pub(crate) fn clean(content: &str) -> String {
        let without_thoughts = without_think_blocks(content);
        let answer = without_thoughts.split_once(THINK_CLOSE).map_or(without_thoughts.as_str(), |(_, after)| after);
        text::trim(answer).to_owned()
    }
}

/// Removes every `<think>` block, lazily up to the next `</think>`; an unclosed one stays, as with the regex in Kotlin.
fn without_think_blocks(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find(THINK_OPEN) {
        let Some(length) = rest[start..].find(THINK_CLOSE) else {
            break;
        };
        result.push_str(&rest[..start]);
        rest = &rest[start + length + THINK_CLOSE.len()..];
    }
    result.push_str(rest);
    result
}
