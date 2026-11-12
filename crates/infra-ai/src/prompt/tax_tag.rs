use dms_domain::text;

const NAMES: [&str; 2] = ["steuer", "tax"];

/// `true` for a tag that marks documents that matter for taxes, such as "Tax" or "Steuer".
pub(crate) fn is_tax_tag(name: &str) -> bool {
    let name = text::lowercase(name);
    NAMES.iter().any(|part| name.contains(part))
}

/// How the prompts name the tag for documents that matter for taxes: the library's own, if it has one.
pub(crate) fn tax_tag_of<'a>(existing_tags: impl IntoIterator<Item = &'a str>) -> String {
    let existing = existing_tags.into_iter().find(|tag| is_tax_tag(tag));
    match existing {
        Some(tag) => format!("\"{tag}\""),
        None => "\"Tax\" (\"Steuer\" if you write German tags)".to_owned(),
    }
}
