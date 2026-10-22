package dms.domain.restructure.template

/** Why a text is no valid [PathTemplateModel]. */
enum class TemplateProblem {
    EMPTY,
    UNCLOSED_BRACE,
    UNKNOWN_PLACEHOLDER,
    INVALID_FIELD_KEY,

    /** A fixed folder name breaks the file name rules, such as `..` or a name with `:`. */
    INVALID_FOLDER_NAME,
}
