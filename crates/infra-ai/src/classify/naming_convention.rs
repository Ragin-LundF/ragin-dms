use dms_domain::text;

const MIN_EXAMPLES: usize = 3;

/// Date prefixes in the order they are checked; `d` is an ASCII digit, `s` one of ` `, `_`, `-`, `!` means "not
/// followed by `-` and a digit".
const DATE_PREFIXES: [(&str, &str); 4] = [
    ("dddd-dd-dd", "start with the document date as YYYY-MM-DD"),
    ("dddd-dd!", "start with year and month as YYYY-MM"),
    ("dddddddd", "start with the document date as YYYYMMDD"),
    ("dddds", "start with the year"),
];
const SEPARATORS: [(&str, &str); 3] = [(" - ", "\" - \""), ("_", "underscores"), (" ", "spaces")];

/// Describes how the library names its files, read from existing file names, so the model's file name fits in.
/// Nothing about the convention is hard-coded; with too few examples it says so.
pub(crate) fn describe(examples: &[String]) -> String {
    let names: Vec<&str> = examples
        .iter()
        .map(|name| name.rsplit_once('.').map_or(name.as_str(), |(base, _)| base))
        .filter(|name| !text::is_blank(name))
        .collect();
    if names.len() < MIN_EXAMPLES {
        return "The library has too few files to show a convention. Use a short, descriptive name.".to_owned();
    }
    let mut parts = Vec::new();
    if let Some((_, description)) =
        DATE_PREFIXES.iter().find(|(pattern, _)| most(&names, |name| starts_with_pattern(name, pattern)))
    {
        parts.push((*description).to_owned());
    }
    if let Some((_, description)) =
        SEPARATORS.iter().find(|(separator, _)| most(&names, |name| name.contains(separator)))
    {
        parts.push(format!("separate their parts with {description}"));
    }
    if parts.is_empty() {
        return "No common pattern; follow the examples.".to_owned();
    }
    format!("Most file names {}.", parts.join(" and "))
}

/// More than half of `names` match.
fn most(names: &[&str], matches: impl Fn(&str) -> bool) -> bool {
    names.iter().filter(|name| matches(name)).count() * 2 > names.len()
}

fn starts_with_pattern(name: &str, pattern: &str) -> bool {
    let mut characters = name.chars();
    for expected in pattern.chars() {
        let ok = match expected {
            '!' => {
                let rest = characters.as_str();
                return !(rest.starts_with('-') && rest[1..].starts_with(|c: char| c.is_ascii_digit()));
            }
            'd' => characters.next().is_some_and(|c| c.is_ascii_digit()),
            's' => characters.next().is_some_and(|c| matches!(c, ' ' | '_' | '-')),
            literal => characters.next() == Some(literal),
        };
        if !ok {
            return false;
        }
    }
    true
}
