use indexmap::IndexMap;

use crate::yaml::{Node, YamlError};

/// Reads the properties of one YAML mapping the way Jackson binds them to a Kotlin data class: a missing or null
/// optional property is `None`, any scalar is accepted as text, numbers and booleans also from text, and a
/// property of the wrong shape is an error.
#[derive(Debug, Clone, Copy)]
pub struct Fields<'a> {
    entries: &'a IndexMap<String, Node>,
}

impl<'a> Fields<'a> {
    /// # Errors
    /// Fails when `node` is no mapping.
    pub fn of(node: &'a Node) -> Result<Self, YamlError> {
        match node {
            Node::Map(entries) => Ok(Self { entries }),
            other => Err(YamlError(format!("expected a mapping, found {other:?}"))),
        }
    }

    /// The property `key`, unless it is missing or null.
    pub fn get(self, key: &str) -> Option<&'a Node> {
        self.entries.get(key).filter(|node| **node != Node::Null)
    }

    /// # Errors
    /// Fails when the property is missing, null or no scalar.
    pub fn text(self, key: &str) -> Result<&'a str, YamlError> {
        self.opt_text(key)?.ok_or_else(|| missing(key))
    }

    /// # Errors
    /// Fails when the property is no scalar.
    pub fn opt_text(self, key: &str) -> Result<Option<&'a str>, YamlError> {
        self.get(key).map(|node| node.scalar_text().ok_or_else(|| wrong(key, "text"))).transpose()
    }

    /// # Errors
    /// Fails when the property is no boolean.
    pub fn opt_bool(self, key: &str) -> Result<Option<bool>, YamlError> {
        self.get(key)
            .map(|node| match node {
                Node::Bool(value) => Ok(*value),
                Node::Text(text) if text.eq_ignore_ascii_case("true") => Ok(true),
                Node::Text(text) if text.eq_ignore_ascii_case("false") => Ok(false),
                _ => Err(wrong(key, "a boolean")),
            })
            .transpose()
    }

    /// # Errors
    /// Fails when the property is missing or no boolean.
    pub fn bool_or(self, key: &str, default: bool) -> Result<bool, YamlError> {
        Ok(self.opt_bool(key)?.unwrap_or(default))
    }

    /// # Errors
    /// Fails when the property is no whole number.
    pub fn opt_i64(self, key: &str) -> Result<Option<i64>, YamlError> {
        self.get(key)
            .map(|node| match node {
                Node::Int(text) | Node::Text(text) => text.trim().parse().map_err(|_| wrong(key, "a whole number")),
                _ => Err(wrong(key, "a whole number")),
            })
            .transpose()
    }

    /// # Errors
    /// Fails when the property is missing or no whole number.
    pub fn i64(self, key: &str) -> Result<i64, YamlError> {
        self.opt_i64(key)?.ok_or_else(|| missing(key))
    }

    /// # Errors
    /// Fails when the property is no whole number in the range of `i32`.
    pub fn opt_i32(self, key: &str) -> Result<Option<i32>, YamlError> {
        self.opt_i64(key)?.map(|value| i32::try_from(value).map_err(|_| wrong(key, "an int"))).transpose()
    }

    /// # Errors
    /// Fails when the property is no number.
    pub fn opt_f64(self, key: &str) -> Result<Option<f64>, YamlError> {
        self.get(key)
            .map(|node| match node {
                Node::Int(text) | Node::Float(text) | Node::Text(text) => {
                    text.trim().parse().map_err(|_| wrong(key, "a number"))
                }
                _ => Err(wrong(key, "a number")),
            })
            .transpose()
    }

    /// The items of a list property; missing or null is an empty list.
    ///
    /// # Errors
    /// Fails when the property is no list.
    pub fn list(self, key: &str) -> Result<&'a [Node], YamlError> {
        match self.get(key) {
            None => Ok(&[]),
            Some(Node::Seq(items)) => Ok(items),
            Some(_) => Err(wrong(key, "a list")),
        }
    }

    /// A list of texts; missing or null is an empty list.
    ///
    /// # Errors
    /// Fails when the property is no list of scalars.
    pub fn texts(self, key: &str) -> Result<Vec<&'a str>, YamlError> {
        self.list(key)?.iter().map(|item| item.scalar_text().ok_or_else(|| wrong(key, "a list of texts"))).collect()
    }

    /// The entries of a mapping property; missing or null is an empty mapping.
    ///
    /// # Errors
    /// Fails when the property is no mapping.
    pub fn entries(self, key: &str) -> Result<Vec<(&'a str, &'a Node)>, YamlError> {
        match self.get(key) {
            None => Ok(Vec::new()),
            Some(Node::Map(entries)) => Ok(entries.iter().map(|(key, value)| (key.as_str(), value)).collect()),
            Some(_) => Err(wrong(key, "a mapping")),
        }
    }

    /// The properties not in `known`, in file order: the fields of a newer version.
    pub fn unknown(self, known: &[&str]) -> IndexMap<String, Node> {
        self.entries
            .iter()
            .filter(|(key, _)| !known.contains(&key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }
}

fn missing(key: &str) -> YamlError {
    YamlError(format!("missing property '{key}'"))
}

fn wrong(key: &str, expected: &str) -> YamlError {
    YamlError(format!("property '{key}' must be {expected}"))
}
