package dms.domain.restructure.template

/** The values a [PathTemplateModel] can insert, by the name written between braces. */
enum class PlaceholderName(val token: String) {
    /** The category's full path, such as `Insurance/Car`; one folder per level. */
    CATEGORY(token = "category"),
    CATEGORY_TOP(token = "category.top"),
    YEAR(token = "year"),
    MONTH(token = "month"),
    CORRESPONDENT(token = "correspondent"),
    KIND(token = "kind"),

    /** A custom field, written `{field:<key>}`. */
    FIELD(token = "field"),
    ;

    companion object {
        fun of(token: String): PlaceholderName? {
            return entries.firstOrNull { it.token == token }
        }
    }
}
