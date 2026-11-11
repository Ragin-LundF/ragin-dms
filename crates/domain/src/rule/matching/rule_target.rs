/// Which part of a suggestion a rule decided; see [`crate::rule::RuleResult::sources`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleTarget {
    Category,
    Tags,
    Fields,
    TargetFolder,
}
