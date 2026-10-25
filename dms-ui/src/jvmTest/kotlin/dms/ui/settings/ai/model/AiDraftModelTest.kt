package dms.ui.settings.ai.model

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
}
