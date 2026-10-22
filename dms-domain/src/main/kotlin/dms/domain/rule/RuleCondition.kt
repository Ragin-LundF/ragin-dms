package dms.domain.rule

/** One condition of a [MatchingRuleModel]; every condition of a rule must hold. */
sealed interface RuleCondition {

    /** The document's correspondent, given or recognized in its text, is [name] (ignoring case). */
    data class CorrespondentIs(val name: String) : RuleCondition {
        init {
            require(value = name.isNotBlank()) { "Correspondent must not be blank" }
        }
    }

    /** The text contains all [words] with [requireAll], otherwise at least one of them (ignoring case). */
    data class TextContains(val words: List<String>, val requireAll: Boolean) : RuleCondition {
        init {
            require(value = words.isNotEmpty() && words.none { it.isBlank() }) { "Words must not be blank" }
        }
    }

    /** The file name matches [glob], where `*` stands for any characters and `?` for one (ignoring case). */
    data class FileNameMatches(val glob: String) : RuleCondition {
        init {
            require(value = glob.isNotBlank()) { "Glob must not be blank" }
        }
    }
}
