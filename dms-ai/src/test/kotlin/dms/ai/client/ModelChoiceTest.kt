package dms.ai.client

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ModelChoiceTest {

    @Test
    fun `embedding models are skipped`() {
        val models = listOf("text-embedding-sample-v1", "multilingual-e5-small", "sample-chat-model")

        assertEquals(expected = "sample-chat-model", actual = ModelChoice.of(models = models))
    }

    @Test
    fun `no chat model gives none`() {
        assertNull(actual = ModelChoice.of(models = listOf("sample-embedding")))
    }
}
