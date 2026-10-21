package dms.ai.client

import java.util.Locale

/**
 * Picks a model when the settings name none. LM Studio can list every downloaded model, including
 * embedding models, which cannot chat; those are skipped.
 */
object ModelChoice {

    private val NOT_FOR_CHAT = listOf("embed", "rerank", "e5-", "bge-", "gte-")

    fun of(models: List<String>): String? {
        return models.firstOrNull { model ->
            val name = model.lowercase(Locale.ROOT)
            NOT_FOR_CHAT.none { it in name }
        }
    }
}
