use crate::yaml::Node;

/// Brings catalog files written by older versions up to [`Self::CURRENT_VERSION`]. Every change of the stored
/// structure that is not just a new optional field bumps the version and adds a step here, with a test that reads a
/// file in the old format.
pub struct CatalogMigrations;

impl CatalogMigrations {
    pub const CURRENT_VERSION: i32 = 1;

    /// The migrated content, or `None` if the file is newer than this version understands or has no version.
    #[must_use]
    pub fn migrate(tree: Node) -> Option<Node> {
        let Node::Map(entries) = &tree else { return None };
        let version: i64 = entries.get("formatVersion")?.scalar_text()?.trim().parse().ok()?;
        (1..=i64::from(Self::CURRENT_VERSION)).contains(&version).then_some(tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::parse;

    fn tree(text: &str) -> Node {
        parse(text).expect("valid YAML").expect("a document")
    }

    #[test]
    fn a_file_of_the_current_version_is_returned_as_it_is() {
        let current = tree("formatVersion: 1\nlibraryId: l\n");

        assert_eq!(CatalogMigrations::migrate(current.clone()), Some(current));
    }

    #[test]
    fn a_file_of_a_newer_version_without_a_version_or_with_version_zero_is_not_read() {
        assert_eq!(CatalogMigrations::migrate(tree("formatVersion: 2\n")), None);
        assert_eq!(CatalogMigrations::migrate(tree("libraryId: l\n")), None);
        assert_eq!(CatalogMigrations::migrate(tree("formatVersion: 0\n")), None);
    }
}
