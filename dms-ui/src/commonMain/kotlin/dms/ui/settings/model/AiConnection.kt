package dms.ui.settings.model

/**
 * The result of the last connection test of the AI settings: the models the server offers, or why
 * it did not answer. [checking] is true while the test runs.
 */
data class AiConnection(
    val checking: Boolean = false,
    val models: List<String> = emptyList(),
    val message: String? = null,
    val isError: Boolean = false,
)
