/// Who made a rule: learned from approved documents and accepted by the user, or written by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleOrigin {
    Learned,
    Manual,
}
