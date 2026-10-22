package dms.domain.rule.model

/** Identifies a matching rule. */
@JvmInline
value class MatchingRuleIdModel(val value: String) {
    init {
        require(value = value.isNotBlank()) { "Rule id must not be blank" }
    }

    override fun toString(): String {
        return value
    }
}
