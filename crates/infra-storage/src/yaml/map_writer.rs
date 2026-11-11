use indexmap::IndexMap;

use crate::yaml::Node;

/// Builds one YAML mapping in property order, with the inclusion rules of Jackson's `@JsonInclude`: `NON_NULL`
/// leaves out `None`, `NON_EMPTY` also empty text and empty lists or mappings.
#[derive(Debug, Default)]
#[must_use]
pub struct MapWriter(IndexMap<String, Node>);

impl MapWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node(mut self, key: &str, node: Node) -> Self {
        self.0.insert(key.to_owned(), node);
        self
    }

    pub fn text(self, key: &str, value: &str) -> Self {
        self.node(key, Node::text(value))
    }

    pub fn bool(self, key: &str, value: bool) -> Self {
        self.node(key, Node::Bool(value))
    }

    pub fn int(self, key: &str, value: impl Into<i64>) -> Self {
        self.node(key, Node::int(value))
    }

    /// `null` when `None` (Jackson's default inclusion).
    pub fn nullable_text(self, key: &str, value: Option<&str>) -> Self {
        self.node(key, value.map_or(Node::Null, Node::text))
    }

    /// `NON_NULL`
    pub fn opt_node(self, key: &str, node: Option<Node>) -> Self {
        match node {
            Some(node) => self.node(key, node),
            None => self,
        }
    }

    /// `NON_NULL`
    pub fn opt_text(self, key: &str, value: Option<&str>) -> Self {
        self.opt_node(key, value.map(Node::text))
    }

    /// `NON_EMPTY`
    pub fn non_empty_text(self, key: &str, value: Option<&str>) -> Self {
        self.opt_text(key, value.filter(|value| !value.is_empty()))
    }

    /// `NON_EMPTY`
    pub fn non_empty_list(self, key: &str, items: Vec<Node>) -> Self {
        if items.is_empty() { self } else { self.node(key, Node::Seq(items)) }
    }

    /// `NON_EMPTY`
    pub fn non_empty_map(self, key: &str, entries: IndexMap<String, Node>) -> Self {
        if entries.is_empty() { self } else { self.node(key, Node::Map(entries)) }
    }

    /// The fields of a newer version, after every known property, as Jackson's `@JsonAnyGetter` writes them.
    pub fn unknown(mut self, unknown: &IndexMap<String, Node>) -> Self {
        for (key, value) in unknown {
            self.0.entry(key.clone()).or_insert_with(|| value.clone());
        }
        self
    }

    pub fn build(self) -> Node {
        Node::Map(self.0)
    }
}
