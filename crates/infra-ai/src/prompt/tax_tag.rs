use dms_domain::text;

const NAMES: [&str; 2] = ["steuer", "tax"];

/// How the prompts name the tag for documents that matter for taxes: the library's own, if it has one.
pub(crate) fn tax_tag_of<'a>(existing_tags: impl IntoIterator<Item = &'a str>) -> String {
    let existing = existing_tags.into_iter().find(|tag| {
        let name = text::lowercase(tag);
        NAMES.iter().any(|part| name.contains(part))
    });
    match existing {
        Some(tag) => format!("\"{tag}\""),
        None => "\"Tax\" (\"Steuer\" if you write German tags)".to_owned(),
    }
}
