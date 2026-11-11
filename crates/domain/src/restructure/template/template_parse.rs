use crate::restructure::{PathTemplate, TemplateProblem};

/// The result of reading a path template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateParse {
    Parsed(PathTemplate),
    Invalid(TemplateProblem),
}
