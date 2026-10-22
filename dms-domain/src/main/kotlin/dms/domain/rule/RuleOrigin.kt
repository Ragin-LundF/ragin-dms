package dms.domain.rule

/** Who made a rule: learned from approved documents and accepted by the user, or written by hand. */
enum class RuleOrigin {
    LEARNED,
    MANUAL,
}
