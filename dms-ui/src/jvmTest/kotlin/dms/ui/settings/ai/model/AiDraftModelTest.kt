package dms.ui.settings.ai.model

import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AiDraftModelTest {

    private val draft = AiDraftModel.of(settings = AiSettingsModel())

    @Test
    fun `the input budget is saved as typed`() {
        val settings = draft.copy(inputCharacters = " 30000 ").toSettings()

        assertEquals(expected = 30_000, actual = settings?.inputCharacters)
    }

    @Test
    fun `an input budget outside its range or not a number is not valid`() {
        listOf("999", "200001", "", "lots").forEach { typed ->
            assertNull(actual = draft.copy(inputCharacters = typed).toSettings(), message = typed)
        }
    }

    @Test
    fun `a draft of saved settings gives them back unchanged`() {
        val saved = AiSettingsModel(model = "sample-model", inputCharacters = 4_000)

        assertEquals(expected = saved, actual = AiDraftModel.of(settings = saved).toSettings())
    }

    @Test
    fun `switching to Anthropic and back keeps the local server and the Claude model`() {
        val local = draft.copy(baseUrl = "http://localhost:11434/v1", model = "sample-model")

        val anthropic = local.copy(provider = AiProvider.ANTHROPIC, anthropicModel = "claude-sonnet-5-5").toSettings()
        val back = anthropic?.let { AiDraftModel.of(settings = it).copy(provider = AiProvider.LOCAL) }?.toSettings()

        assertEquals(expected = AiProvider.ANTHROPIC, actual = anthropic?.provider)
        assertEquals(expected = "http://localhost:11434/v1", actual = back?.baseUrl)
        assertEquals(expected = "sample-model", actual = back?.model)
        assertEquals(expected = "claude-sonnet-5-5", actual = back?.anthropicModel)
    }

    @Test
    fun `a blank Claude model saves the default one`() {
        val settings = draft.copy(provider = AiProvider.ANTHROPIC, anthropicModel = " ").toSettings()

        assertEquals(expected = AiSettingsModel.DEFAULT_ANTHROPIC_MODEL, actual = settings?.anthropicModel)
    }

    @Test
    fun `a bad local address only blocks saving while Local AI is chosen, and the problem names the field`() {
        val broken = draft.copy(baseUrl = "localhost")

        assertEquals(
            expected = "Enter a server address starting with http:// or https://",
            actual = broken.problem(),
        )
        assertNull(actual = broken.toSettings())
        assertNull(actual = broken.copy(provider = AiProvider.ANTHROPIC).problem())
        assertEquals(
            expected = "Enter a timeout above 0 seconds",
            actual = draft.copy(provider = AiProvider.ANTHROPIC, timeoutSeconds = "0").problem(),
        )
    }
}
